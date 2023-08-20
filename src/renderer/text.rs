use fontdue::layout::{
    CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle, VerticalAlign,
};
use glam::Vec2;
use wgpu::TextureFormat;

use crate::{
    arena::ArenaId, components::color::Color, utils::float_ord::FloatOrd, Rect, Transform,
};

use super::{
    font_atlas::FontAtlas,
    fonts::{Font, GlyphAtlasInfo, PositionedGlyph},
    texture::Texture,
    Renderer,
};

pub struct Text {
    pub handle: ArenaId<Font>,
    pub value: String,
    font_size: f32,
    pub color: Color,
    vertical_alignment: VerticalAlign,
    horizontal_alignment: HorizontalAlign,
    y_axis_orientation: CoordinateSystem,
}

impl Text {
    pub fn new(value: &str, font_size: f32) -> Self {
        Self {
            value: value.to_owned(),
            font_size,
            ..Default::default()
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            handle: ArenaId::first(),
            value: Default::default(),
            font_size: Default::default(),
            vertical_alignment: VerticalAlign::Top,
            horizontal_alignment: HorizontalAlign::Left,
            y_axis_orientation: CoordinateSystem::PositiveYUp,
            color: Color::WHITE, // White
        }
    }
}

pub(crate) fn get_text_layout_offset_x(text_layout: &Layout) -> f32 {
    let offset_x = text_layout
        .glyphs()
        .iter()
        .map(|glyph| glyph.x)
        .min_by(|a, b| a.total_cmp(b))
        .unwrap_or_default();

    offset_x
}

pub(crate) fn get_text_layout_size(text_layout: &Layout) -> Vec2 {
    let height = text_layout
        .lines()
        .iter()
        .flat_map(|line_pos_vec| line_pos_vec.iter())
        .map(|line| line.baseline_y - line.min_descent)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or_default();

    let width = text_layout
        .glyphs()
        .iter()
        .map(|glyph| glyph.x + glyph.width as f32)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or_default();

    Vec2::new(width, height)
}

impl Renderer {
    pub(crate) fn paint_text(&mut self, text: &Text) -> Vec<PositionedGlyph> {
        self.add_glyphs_to_atlas(&text, text.font_size);

        let texture_handle = *self
            .fonts
            .get(text.handle)
            .unwrap()
            .texture_ids
            .get(&(FloatOrd(text.font_size)))
            .expect("Error, missing texture id for font");

        let mut text_layout = Layout::new(CoordinateSystem::PositiveYUp);

        text_layout.reset(&LayoutSettings {
            x: 0.0,
            y: 0.0,
            max_width: None,
            max_height: None,
            horizontal_align: text.horizontal_alignment,
            vertical_align: text.vertical_alignment,
            ..Default::default()
        });
        let font = &self.fonts.get(text.handle).unwrap().font;
        text_layout.append(&[font], &TextStyle::new(&text.value, text.font_size, 0));

        let offset_x = get_text_layout_offset_x(&text_layout);

        let pos = Vec2::new(offset_x, 0.0);

        text_layout
            .glyphs()
            .iter()
            .map(|glyph| {
                let atlas_info = self
                    .get_glyph_atlas_info(text.font_size, text.handle, glyph.parent, texture_handle)
                    .unwrap();

                let size = Vec2::new(glyph.width as f32, glyph.height as f32);
                return PositionedGlyph {
                    position: pos + Vec2::new(glyph.x, glyph.y),

                    rect: Rect::from_center_size(pos + Vec2::new(glyph.x, glyph.y), size),
                    atlas_info,
                };
            })
            .collect()
    }

    // pub(crate) fn get_positioned_glyphs(
    //     &mut self,
    //     text: &Text,
    //     container_size: Option<Vec2>,
    // ) -> Vec<PositionedGlyph> {
    //     self.add_glyphs_to_atlas(&text, text.font_size);

    //     let texture_handle = *self
    //         .fonts
    //         .get(text.handle)
    //         .unwrap()
    //         .texture_ids
    //         .get(&(FloatOrd(text.font_size)))
    //         .expect("Error, missing texture id for font");

    //     let mut positioned_glyphs = Vec::new();

    //     let mut layout = Layout::new(text.y_axis_orientation);

    //     layout.reset(&LayoutSettings {
    //         x: 0.0,
    //         y: 0.0,
    //         max_width: container_size.map(|size| size.x),
    //         max_height: container_size.map(|size| size.y),
    //         horizontal_align: text.horizontal_alignment,
    //         vertical_align: text.vertical_alignment,
    //         ..Default::default()
    //     });
    //     let font = &self.fonts.get(text.handle).unwrap().font;
    //     layout.append(&[font], &TextStyle::new(&text.value, text.font_size, 0));

    //     for glyph in layout.glyphs() {
    //         let atlas_info = self
    //             .get_glyph_atlas_info(text.font_size, text.handle, glyph.parent, texture_handle)
    //             .unwrap();

    //         positioned_glyphs.push(PositionedGlyph {
    //             position: Vec2::new(glyph.x, glyph.y),
    //             rect: atlas_info.texture_rect,
    //             atlas_info,
    //         });
    //     }

    //     positioned_glyphs
    // }

    pub(crate) fn add_glyphs_to_atlas(&mut self, text: &Text, font_size: f32) {
        let font_atlas = self
            .font_atlases
            .entry((FloatOrd(font_size), text.handle))
            .or_insert_with(|| FontAtlas::new(Vec2::splat(512.0)));
        let font = self.fonts.get(text.handle).unwrap();
        let mut update_texture_data = None;
        for character in text.value.chars() {
            if !font_atlas.has_glyph(character) {
                let (metrics, bitmap) = font.rasterize(character, font_size);
                update_texture_data = font_atlas.add_glyph(character, &bitmap, metrics);
            }
        }

        if let Some(texture) = update_texture_data {
            let texture = self.add_texture_bytes(
                &texture.data,
                texture.dimensions,
                crate::texture::TextureSamplerType::Linear,
                TextureFormat::Rgba8UnormSrgb,
            );

            if let Some(handle) = self
                .fonts
                .get(text.handle)
                .unwrap()
                .texture_ids
                .get(&(FloatOrd(text.font_size)))
            {
                *self.textures.get_mut(*handle).unwrap() = texture;
            } else {
                let texture_handle = self.textures.insert(texture);

                self.fonts
                    .get_mut(text.handle)
                    .unwrap()
                    .texture_ids
                    .insert(FloatOrd(text.font_size), texture_handle);
            }
        }
    }

    pub fn get_glyph_atlas_info(
        &mut self,
        font_size: f32,
        font_id: ArenaId<Font>,
        glyph: char,
        texture_handle: ArenaId<Texture>,
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
                    texture_handle,
                    atlas_size: texture_atlas.size,
                });
        }

        None
    }
}
