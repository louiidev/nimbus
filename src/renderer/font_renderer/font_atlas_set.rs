use std::collections::HashMap;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use glam::Vec2;

use crate::{
    areana::ArenaId,
    components::{
        dynamic_texture_atlas_builder::TempImageData, rect::Rect, text::Text,
        texture_atlas::TextureAtlas,
    },
    utils::float_ord::FloatOrd,
};

use super::{
    font::{Font, FontSizeKey, GlyphAtlasInfo, PositionedGlyph},
    font_atlas::FontAtlas,
};

pub(crate) struct FontAtlasSet {}

impl Default for FontAtlasSet {
    fn default() -> Self {
        FontAtlasSet {
            font_atlases: HashMap::with_capacity_and_hasher(1, Default::default()),
        }
    }
}

impl FontAtlasSet {
    pub fn has_glyph(&self, glyph: char, font_id: ArenaId, font_size: f32) -> bool {
        self.font_atlases
            .get(&(FloatOrd(font_size), font_id))
            .unwrap()
            .has_glyph(glyph)
    }

    pub fn add_glyphs_to_atlas(
        &mut self,
        font: &Font,
        font_id: ArenaId,
        text: &str,
        font_size: f32,
    ) -> Option<TempImageData> {
        let font_atlas = self
            .font_atlases
            .entry((FloatOrd(font_size), font_id))
            .or_insert_with(|| FontAtlas::new(Vec2::splat(512.0)));

        let mut update_texture_data = None;
        for character in text.chars() {
            if !font_atlas.has_glyph(character) {
                let (metrics, bitmap) = font.rasterize(character, font_size);
                update_texture_data = font_atlas.add_glyph(character, &bitmap, metrics);
            }
        }

        update_texture_data
    }

    pub fn queue_text(
        &mut self,
        font: &Font,
        text: &Text,
        container: &Rect,
        y_axis_orientation: CoordinateSystem,
    ) -> Vec<PositionedGlyph> {
        self.add_glyphs_to_atlas(&font, text.font_id, &text.value, text.theme.font_size);

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
                .get_glyph_atlas_info(text.theme.font_size, text.font_id, glyph.parent)
                .unwrap();

            positioned_glyphs.push(PositionedGlyph {
                position: Vec2::new(glyph.x, glyph.y),
                rect: atlas_info.texture_rect,
                atlas_info,
            });
        }

        positioned_glyphs
    }

    pub fn get_glyph_atlas_info(
        &mut self,
        font_size: f32,
        font_id: ArenaId,
        glyph: char,
    ) -> Option<GlyphAtlasInfo> {
        let atlas = self.font_atlases.get(&(FloatOrd(font_size), font_id));

        if let Some(font_atlas) = atlas {
            let texture_atlas = &font_atlas.dynamic_texture_atlas_builder.texture_atlas;

            return font_atlas
                .get_glyph_index(glyph)
                .map(|(glyph_index, metrics)| GlyphAtlasInfo {
                    texture_rect: texture_atlas.textures.get(glyph_index).copied().unwrap(),
                    metrics,
                });
        }

        None
    }
}
