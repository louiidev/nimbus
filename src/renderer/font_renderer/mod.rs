pub mod font;
mod font_atlas;
// mod font_atlas_set;

use std::collections::HashMap;

use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    FontResult,
};
use glam::Vec2;
use wgpu::{Device, Queue};

use self::{
    font::{Font, FontSizeKey, GlyphAtlasInfo, PositionedGlyph},
    font_atlas::FontAtlas,
};
use crate::{
    arena::{Arena, ArenaId},
    components::{dynamic_texture_atlas_builder::TempImageData, text::Text},
    utils::float_ord::FloatOrd,
};

use super::texture::Texture;

pub struct FontRenderer {
    pub(crate) fonts: Arena<Font>,
    font_atlases: HashMap<(FontSizeKey, ArenaId), FontAtlas>,
    pub(crate) texture_mapping: HashMap<(FontSizeKey, ArenaId), ArenaId>,
}

impl FontRenderer {
    pub(crate) fn new() -> FontRenderer {
        let bytes = include_bytes!("../../assets/fonts/Roboto-Regular.ttf");

        let mut font_renderer = FontRenderer {
            fonts: Arena::new(),
            font_atlases: HashMap::default(),
            texture_mapping: HashMap::default(),
        };

        font_renderer
            .load_font(bytes)
            .expect("Couldn't load default font");

        font_renderer
    }

    pub fn queue_text(
        &mut self,
        text: &Text,
        container_size: Option<Vec2>,
        y_axis_orientation: CoordinateSystem,
        textures: &mut Arena<Texture>,
        device: &Device,
        queue: &Queue,
    ) -> Vec<PositionedGlyph> {
        let texture = self.add_glyphs_to_atlas(text.font_id, &text.value, text.theme.font_size);

        if let Some(temp_texture_data) = texture {
            let updated_texture = Texture::from_detailed_bytes(
                device,
                queue,
                &temp_texture_data.data,
                (temp_texture_data.size.x as _, temp_texture_data.size.y as _),
            );

            if let Some(texture_id) = self
                .texture_mapping
                .get(&(FloatOrd(text.theme.font_size), text.font_id))
            {
                *textures.get_mut(*texture_id).unwrap() = updated_texture;
            } else {
                let texture_id = textures.insert(updated_texture);

                self.texture_mapping
                    .insert((FloatOrd(text.theme.font_size), text.font_id), texture_id);
            }
        }

        let mut positioned_glyphs = Vec::new();

        let mut layout = Layout::new(y_axis_orientation);

        layout.reset(&LayoutSettings {
            x: 0.0,
            y: 0.0,
            max_width: container_size.map(|size| size.x),
            max_height: container_size.map(|size| size.y),
            horizontal_align: text.alignment.horizontal,
            vertical_align: text.alignment.vertical,
            ..Default::default()
        });
        let font = self.fonts.get(text.font_id).unwrap();
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

    pub(crate) fn add_glyphs_to_atlas(
        &mut self,
        font_id: ArenaId,
        text: &str,
        font_size: f32,
    ) -> Option<TempImageData> {
        let font_atlas = self
            .font_atlases
            .entry((FloatOrd(font_size), font_id))
            .or_insert_with(|| FontAtlas::new(Vec2::splat(512.0)));
        let font = self.fonts.get(font_id).unwrap();
        let mut update_texture_data = None;
        for character in text.chars() {
            if !font_atlas.has_glyph(character) {
                let (metrics, bitmap) = font.rasterize(character, font_size);
                update_texture_data = font_atlas.add_glyph(character, &bitmap, metrics);
            }
        }

        update_texture_data
    }

    pub fn load_font_with_id(&mut self, font_data: &[u8], id: ArenaId) -> FontResult<()> {
        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())?;

        let font_to_replace = self
            .fonts
            .get_mut(id)
            .expect("Trying to replace font id that doesnt exist");
        *font_to_replace = Font { font };

        Ok(())
    }

    pub fn load_font(&mut self, font_data: &[u8]) -> FontResult<ArenaId> {
        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())?;
        Ok(self.fonts.insert(Font { font }))
    }

    pub fn get_glyph_atlas_info(
        &mut self,
        font_size: f32,
        font_id: ArenaId,
        glyph: char,
    ) -> Option<GlyphAtlasInfo> {
        let key = (FloatOrd(font_size), font_id);
        let atlas = self.font_atlases.get(&key);

        if let Some(font_atlas) = atlas {
            let texture_atlas = &font_atlas.dynamic_texture_atlas_builder.texture_atlas;

            return font_atlas
                .get_glyph_index(glyph)
                .map(|(glyph_index, metrics)| GlyphAtlasInfo {
                    texture_rect: texture_atlas.textures.get(glyph_index).copied().unwrap(),
                    metrics,
                    texture_id: *self.texture_mapping.get(&key).unwrap(),
                    atlas_size: texture_atlas.size,
                });
        }

        None
    }
}
