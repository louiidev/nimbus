use hashbrown::HashMap;

use ab_glyph::{Font as _, FontArc, ScaleFont as _};
use ab_glyph::{GlyphId, OutlinedGlyph, PxScale};
use glam::{Vec2, Vec3};
use glyph_brush_layout::{
    FontId, GlyphPositioner, HorizontalAlign, Layout, SectionGeometry, SectionGlyph, SectionText,
    VerticalAlign,
};

use wgpu::{Extent3d, TextureDimension, TextureFormat};

use crate::font::{Font, FontSizeKey, GlyphAtlasInfo, PositionedGlyph};
use crate::rect::Rect;
use crate::transform::Transform;
use crate::{
    components::text::Text,
    internal_image::Image,
    resources::utils::Assets,
    texture_atlas::{DynamicTextureAtlasBuilder, TextureAtlas},
    utils::float_ord::FloatOrd,
};

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

pub struct FontAtlasSet {
    font_atlases: HashMap<FontSizeKey, FontAtlas>,
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
            .unwrap()
            .has_glyph(glyph_id)
    }

    pub fn queue_text(
        &mut self,
        font: &FontArc,
        text: &Text,
        container: Rect,
        font_size: f32,
        texture_atlases: &mut Assets<TextureAtlas>,
        temp_image_storage: &mut Assets<Image>,
    ) -> Vec<PositionedGlyph> {
        let geom = SectionGeometry {
            bounds: (container.max.x, container.max.y),
            // screen_position: (container.min.x, container.min.y),
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

        let mut min_x: f32 = std::f32::MAX;
        let mut min_y: f32 = std::f32::MAX;
        let mut max_x: f32 = std::f32::MIN;
        let mut max_y: f32 = std::f32::MIN;

        for sg in &section_glyphs {
            let glyph = &sg.glyph;
            min_x = min_x.min(glyph.position.x);
            min_y = min_y.min(glyph.position.y - scaled_font.ascent());
            max_x = max_x.max(glyph.position.x + scaled_font.h_advance(glyph.id));
            max_y = max_y.max(glyph.position.y - scaled_font.descent());
        }

        let size = Vec2::new(max_x - min_x, max_y - min_y);

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

            if let Some(outlined_glyph) = font.outline_glyph(glyph) {
                let bounds = outlined_glyph.px_bounds();
                let atlas_info = self
                    .get_glyph_atlas_info(font_size, glyph_id)
                    .map(Ok)
                    .unwrap_or_else(|| {
                        self.add_glyph_to_atlas(outlined_glyph, texture_atlases, temp_image_storage)
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

                let height = container.height();
                let width = container.width();

                let alignment_offset = match text.alignment.vertical {
                    VerticalAlign::Top => Vec3::new(0.0, -height, 0.0),
                    VerticalAlign::Center => Vec3::new(0.0, -height * 0.5, 0.0),
                    VerticalAlign::Bottom => Vec3::ZERO,
                } + match text.alignment.horizontal {
                    HorizontalAlign::Left => Vec3::ZERO,
                    HorizontalAlign::Center => Vec3::new(-width * 0.5, 0.0, 0.0),
                    HorizontalAlign::Right => Vec3::new(-width, 0.0, 0.0),
                };

                let glyph_transform =
                    Transform::from_translation(alignment_offset + position.extend(0.));
                // NOTE: Should match `bevy_ui::render::extract_text_uinodes`

                positioned_glyphs.push(PositionedGlyph {
                    position,
                    size,
                    atlas_info,
                    section_index: sg.section_index,
                    byte_index,
                    glyph_transform,
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
                FontAtlas::new(temp_image_storage, texture_atlases, Vec2::splat(512.0))
            });

        let glyph_texture = Font::get_outlined_glyph_texture(&outlined_glyph);

        if !font_atlases.add_glyph(
            temp_image_storage,
            texture_atlases,
            glyph.id,
            &glyph_texture,
        ) {
            return Err(TextError::FailedToAddGlyph(glyph.id));
        }
        Ok(self.get_glyph_atlas_info(font_size, glyph.id).unwrap())
    }

    pub fn get_glyph_atlas_info(
        &mut self,
        font_size: f32,
        glyph_id: GlyphId,
    ) -> Option<GlyphAtlasInfo> {
        let atlas = self.font_atlases.get(&FloatOrd(font_size));

        if let Some(atlas) = atlas {
            return atlas
                .get_glyph_index(glyph_id)
                .map(|glyph_index| GlyphAtlasInfo {
                    texture_atlas_id: atlas.texture_atlas_id,
                    glyph_index,
                });
        }

        None
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TextError {
    NoSuchFont,
    FailedToAddGlyph(GlyphId),
}
