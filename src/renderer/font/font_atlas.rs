use std::collections::HashMap;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::Metrics;
use glam::Vec2;

use wgpu::{Extent3d, TextureDimension, TextureFormat};

use crate::areana::{Arena, ArenaId};
use crate::components::texture_atlas::{DynamicTextureAtlasBuilder, TextureAtlas};

pub struct FontAtlas {
    pub dynamic_texture_atlas_builder: DynamicTextureAtlasBuilder,
    pub glyph_atlas_info: HashMap<char, (usize, Metrics)>,
    pub texture_atlas_id: ArenaId,
}

impl FontAtlas {
    pub fn new(
        images: &mut Assets<Image>,
        texture_atlases: &mut Arena<TextureAtlas>,
        size: Vec2,
    ) -> FontAtlas {
        images.add(Image::new_fill(
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
        let atlas_texture_id = texture_atlases.insert(texture_atlas);

        Self {
            texture_atlas_id: atlas_texture_id,
            glyph_atlas_info: HashMap::default(),
            dynamic_texture_atlas_builder: DynamicTextureAtlasBuilder::new(size, 1),
        }
    }

    pub fn get_glyph_index(&self, glyph_id: char) -> Option<(usize, Metrics)> {
        self.glyph_atlas_info.get(&glyph_id).copied()
    }

    pub fn has_glyph(&self, character: char) -> bool {
        self.glyph_atlas_info.contains_key(&character)
    }

    pub fn add_glyph(
        &mut self,
        images: &mut Assets<Image>,
        texture_atlases: &mut Arena<TextureAtlas>,
        glyph: char,
        image: &Image,
        glyph_metrics: Metrics,
    ) {
        let texture_atlas = texture_atlases.get_mut(&self.texture_atlas_id).unwrap();

        if let Some(index) =
            self.dynamic_texture_atlas_builder
                .add_texture(texture_atlas, images, image)
        {
            self.glyph_atlas_info.insert(glyph, (index, glyph_metrics));
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
    pub fn has_glyph(&self, glyph: char, font_size: f32) -> bool {
        self.font_atlases
            .get(&FloatOrd(font_size))
            .unwrap()
            .has_glyph(glyph)
    }

    pub fn add_glyphs_to_atlas(
        &mut self,
        font: &FontData,
        texture_atlases: &mut Assets<TextureAtlas>,
        images: &mut Assets<Image>,
        text: &str,
        font_size: f32,
    ) {
        let font_atlas = self
            .font_atlases
            .entry(FloatOrd(font_size))
            .or_insert_with(|| FontAtlas::new(images, texture_atlases, Vec2::splat(512.0)));

        for character in text.chars() {
            if !font_atlas.has_glyph(character) {
                let (metrics, bitmap) = font.rasterize(character, font_size);
                font_atlas.add_glyph(images, texture_atlases, character, &bitmap, metrics);
            }
        }
    }

    pub fn queue_text(
        &mut self,
        font: &FontData,
        text: &Text,
        container: &Rect,
        texture_atlases: &mut Assets<TextureAtlas>,
        temp_image_storage: &mut Assets<Image>,
        y_axis_orientation: CoordinateSystem,
    ) -> Vec<PositionedGlyph> {
        self.add_glyphs_to_atlas(
            &font,
            texture_atlases,
            temp_image_storage,
            &text.value,
            text.theme.font_size,
        );

        let mut positioned_glyphs = Vec::new();

        let mut layout = Layout::new(y_axis_orientation);

        layout.reset(&LayoutSettings {
            x: 0.0,
            y: 0.0,
            max_width: Some(container.max.x - container.min.x),
            max_height: Some(container.max.y - container.min.y),
            horizontal_align: text.alignment.horizontal,
            vertical_align: text.alignment.vertical,
            ..Default::default()
        });

        layout.append(
            &[&font.font],
            &TextStyle::new(&text.value, text.theme.font_size, 0),
        );

        for glyph in layout.glyphs() {
            let atlas_info = self
                .get_glyph_atlas_info(text.theme.font_size, glyph.parent)
                .unwrap();
            let texture_atlas = texture_atlases.get(&atlas_info.texture_atlas_id).unwrap();

            positioned_glyphs.push(PositionedGlyph {
                position: Vec2::new(glyph.x, glyph.y),
                rect: texture_atlas.textures[atlas_info.glyph_index],
                atlas_info,
            });
        }

        positioned_glyphs
    }

    pub fn get_glyph_atlas_info(&mut self, font_size: f32, glyph: char) -> Option<GlyphAtlasInfo> {
        let atlas = self.font_atlases.get(&FloatOrd(font_size));

        if let Some(atlas) = atlas {
            return atlas
                .get_glyph_index(glyph)
                .map(|(glyph_index, metrics)| GlyphAtlasInfo {
                    texture_atlas_id: atlas.texture_atlas_id,
                    glyph_index,
                    metrics,
                });
        }

        None
    }
}
