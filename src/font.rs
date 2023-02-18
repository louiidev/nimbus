use ab_glyph::{Font as _, FontArc, ScaleFont as _};
use ab_glyph::{FontVec, GlyphId, InvalidFont, OutlinedGlyph, PxScale};
use glam::Vec2;
use glyph_brush_layout::{
    FontId, GlyphPositioner, Layout, SectionGeometry, SectionGlyph, SectionText,
};
use hashbrown::HashMap;
use wgpu::{Extent3d, TextureDimension, TextureFormat};

use crate::{
    components::text::Text,
    internal_image::Image,
    resources::utils::Assets,
    texture_atlas::{DynamicTextureAtlasBuilder, TextureAtlas},
    utils::float_ord::FloatOrd,
};

type FontSizeKey = FloatOrd;

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub position: Vec2,
    pub size: Vec2,
    pub atlas_info: GlyphAtlasInfo,
    pub section_index: usize,
    pub byte_index: usize,
}

pub struct Font {
    pub font: FontArc,
}

impl Font {
    pub fn try_from_bytes(font_data: Vec<u8>) -> Result<Self, InvalidFont> {
        let font = FontVec::try_from_vec(font_data)?;
        let font = FontArc::new(font);
        Ok(Font { font })
    }

    pub fn get_outlined_glyph_texture(outlined_glyph: &OutlinedGlyph) -> Image {
        let bounds = outlined_glyph.px_bounds();
        let width = bounds.width() as usize;
        let height = bounds.height() as usize;
        let mut alpha = vec![0.0; width * height];
        outlined_glyph.draw(|x, y, v| {
            alpha[y as usize * width + x as usize] = v;
        });

        // TODO: make this texture grayscale
        let image = Image::new(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            alpha
                .iter()
                .flat_map(|a| vec![255, 255, 255, (*a * 255.0) as u8])
                .collect::<Vec<u8>>(),
            TextureFormat::Rgba8UnormSrgb,
        );

        image
    }
}

pub struct FontAtlas {
    pub dynamic_texture_atlas_builder: DynamicTextureAtlasBuilder,
    pub glyph_to_atlas_index: HashMap<GlyphId, usize>,
    pub texture_atlas_id: uuid::Uuid,
}

impl FontAtlas {
    pub fn new(
        images: &mut Assets<Image>,
        texture_atlases: &mut Assets<TextureAtlas>,
        size: Vec2,
    ) -> FontAtlas {
        let atlas_texture_id = images.add(Image::new_fill(
            Extent3d {
                width: size.x as u32,
                height: size.y as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
        ));

        let texture_atlas = TextureAtlas::new_empty(atlas_texture_id, size);
        texture_atlases.insert(atlas_texture_id, texture_atlas);

        Self {
            texture_atlas_id: atlas_texture_id,
            glyph_to_atlas_index: HashMap::default(),
            dynamic_texture_atlas_builder: DynamicTextureAtlasBuilder::new(size, 1),
        }
    }

    pub fn get_glyph_index(&self, glyph_id: GlyphId) -> Option<usize> {
        self.glyph_to_atlas_index.get(&glyph_id).copied()
    }

    pub fn has_glyph(&self, glyph_id: GlyphId) -> bool {
        self.glyph_to_atlas_index.contains_key(&glyph_id)
    }

    pub fn add_glyph(
        &mut self,
        images: &mut Assets<Image>,
        texture_atlases: &mut Assets<TextureAtlas>,
        glyph_id: GlyphId,
        image: &Image,
    ) -> bool {
        let texture_atlas = texture_atlases.get_mut(&self.texture_atlas_id).unwrap();
        if let Some(index) =
            self.dynamic_texture_atlas_builder
                .add_texture(texture_atlas, images, image)
        {
            self.glyph_to_atlas_index.insert(glyph_id, index);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlyphAtlasInfo {
    pub texture_atlas_id: uuid::Uuid,
    pub glyph_index: usize,
}

pub struct FontAtlasSet {
    font_atlases: HashMap<FontSizeKey, Vec<FontAtlas>>,
}

impl Default for FontAtlasSet {
    fn default() -> Self {
        FontAtlasSet {
            font_atlases: HashMap::with_capacity_and_hasher(1, Default::default()),
        }
    }
}

impl FontAtlasSet {
    pub fn has_glyph(&self, glyph_id: GlyphId, font_size: f32) -> bool {
        self.font_atlases
            .get(&FloatOrd(font_size))
            .map_or(false, |font_atlas| {
                font_atlas.iter().any(|atlas| atlas.has_glyph(glyph_id))
            })
    }

    pub fn queue_text(
        &mut self,
        font: &FontArc,
        text: &Text,
        bounds: Vec2,
        font_size: f32,
        texture_atlases: &mut Assets<TextureAtlas>,
        temp_image_storage: &mut Assets<Image>,
    ) -> Vec<PositionedGlyph> {
        let geom = SectionGeometry {
            bounds: (bounds.x, bounds.y),
            ..Default::default()
        };

        let section = SectionText {
            font_id: FontId(0),
            scale: PxScale::from(font_size),
            text: &text.value,
        };

        let section_glyphs = Layout::default()
            .h_align(text.alignment.horizontal)
            .v_align(text.alignment.vertical)
            .calculate_glyphs(&[font], &geom, &[section]);

        let scaled_font = ab_glyph::Font::as_scaled(&font, font_size);

        let mut min_x = std::f32::MAX;
        let mut min_y = std::f32::MAX;
        let mut max_y = std::f32::MIN;
        for sg in &section_glyphs {
            let glyph = &sg.glyph;

            min_x = min_x.min(glyph.position.x);
            min_y = min_y.min(glyph.position.y - scaled_font.ascent());
            max_y = max_y.max(glyph.position.y - scaled_font.descent());
        }
        min_x = min_x.floor();
        min_y = min_y.floor();
        max_y = max_y.floor();

        let mut positioned_glyphs = Vec::new();

        for sg in section_glyphs {
            let SectionGlyph {
                section_index: _,
                byte_index,
                mut glyph,
                font_id: _,
            } = sg;
            let glyph_id = glyph.id;

            let glyph_x = glyph.position.x.round();
            glyph.position.x = 0.;
            glyph.position.y = glyph.position.y.ceil();
            let outline_glyph = font.outline_glyph(glyph.clone()).unwrap();

            if let Some(outlined_glyph) = font.outline_glyph(glyph) {
                let bounds = outlined_glyph.px_bounds();
                let atlas_info = self
                    .get_glyph_atlas_info(font_size, glyph_id)
                    .map(Ok)
                    .unwrap_or_else(|| {
                        self.add_glyph_to_atlas(outline_glyph, texture_atlases, temp_image_storage)
                    })
                    .unwrap();
                let texture_atlas = texture_atlases.get(&atlas_info.texture_atlas_id).unwrap();
                let glyph_rect = texture_atlas.textures[atlas_info.glyph_index];
                let size = Vec2::new(glyph_rect.width(), glyph_rect.height());
                let x = bounds.min.x + size.x / 2.0 - min_x;

                // let y = match y_axis_orientation {
                //     YAxisOrientation::BottomToTop => max_y - bounds.max.y + size.y / 2.0,
                //     YAxisOrientation::TopToBottom => bounds.min.y + size.y / 2.0 - min_y,
                // };

                let y = bounds.min.y + size.y / 2.0 - min_y;

                let position = Vec2::new(glyph_x, 0.) + Vec2::new(x, y);

                positioned_glyphs.push(PositionedGlyph {
                    position,
                    size,
                    atlas_info,
                    section_index: sg.section_index,
                    byte_index,
                });
            }
        }

        positioned_glyphs
    }

    pub fn add_glyph_to_atlas(
        &mut self,
        outlined_glyph: OutlinedGlyph,
        texture_atlases: &mut Assets<TextureAtlas>,
        temp_image_storage: &mut Assets<Image>,
    ) -> Result<GlyphAtlasInfo, TextError> {
        let glyph = outlined_glyph.glyph(); // FUTURE CHECK used to be shared ref, why can't we clone?

        let font_size = glyph.scale.y;

        let font_atlases = self
            .font_atlases
            .entry(FloatOrd(font_size))
            .or_insert_with(|| {
                vec![FontAtlas::new(
                    temp_image_storage,
                    texture_atlases,
                    Vec2::splat(512.0),
                )]
            });

        let glyph_texture = Font::get_outlined_glyph_texture(&outlined_glyph);

        let add_char_to_font_atlas = |atlas: &mut FontAtlas| -> bool {
            atlas.add_glyph(
                temp_image_storage,
                texture_atlases,
                glyph.id,
                &glyph_texture,
            )
        };

        if !font_atlases.iter_mut().any(add_char_to_font_atlas) {
            let glyph_max_size: u32 = glyph_texture
                .texture_descriptor
                .size
                .height
                .max(glyph_texture.texture_descriptor.size.width);
            // Pick the higher  of 512 or the smallest power of 2 greater than glyph_max_size
            let containing = (1u32 << (32 - glyph_max_size.leading_zeros())).max(512) as f32;
            font_atlases.push(FontAtlas::new(
                temp_image_storage,
                texture_atlases,
                Vec2::new(containing, containing),
            ));
            if !font_atlases.last_mut().unwrap().add_glyph(
                temp_image_storage,
                texture_atlases,
                glyph.id,
                &glyph_texture,
            ) {
                return Err(TextError::FailedToAddGlyph(glyph.id));
            }
        }
        Ok(self.get_glyph_atlas_info(font_size, glyph.id).unwrap())
    }

    pub fn get_glyph_atlas_info(
        &mut self,
        font_size: f32,
        glyph_id: GlyphId,
    ) -> Option<GlyphAtlasInfo> {
        self.font_atlases
            .get(&FloatOrd(font_size))
            .and_then(|font_atlases| {
                font_atlases
                    .iter()
                    .find_map(|atlas| {
                        atlas
                            .get_glyph_index(glyph_id)
                            .map(|glyph_index| (glyph_index, atlas.texture_atlas_id))
                    })
                    .map(|(glyph_index, texture_atlas_id)| GlyphAtlasInfo {
                        texture_atlas_id,
                        glyph_index,
                    })
            })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TextError {
    NoSuchFont,
    FailedToAddGlyph(GlyphId),
}
