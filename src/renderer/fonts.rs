use crate::arena::ArenaId;

use crate::utils::float_ord::FloatOrd;
use core::hash::{Hash, Hasher};
use fontdue::{Font as ExternalFont, FontResult, Metrics};
use glam::Vec2;
use std::collections::HashMap;
use wgpu::TextureFormat;

use super::{
    rect::Rect,
    texture::{Image, Texture},
    Renderer,
};

pub type FontSizeKey = FloatOrd;

#[derive(Debug, Clone)]
pub struct GlyphAtlasInfo {
    pub texture_rect: Rect,
    pub metrics: Metrics,
    pub texture_handle: ArenaId<Texture>,
    pub atlas_size: Vec2,
}

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub position: Vec2,
    pub rect: Rect,
    pub atlas_info: GlyphAtlasInfo,
}

pub struct Font {
    pub font: ExternalFont,
    pub texture_ids: HashMap<FloatOrd, ArenaId<Texture>>,
}

impl Hash for Font {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font.hash(state);
    }
}

impl Font {
    pub fn try_from_bytes(font_data: &[u8]) -> FontResult<Self> {
        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())?;
        Ok(Font {
            font,
            texture_ids: HashMap::default(),
        })
    }

    pub(crate) fn rasterize(&self, character: char, font_size: f32) -> (Metrics, Image) {
        let (metrics, bitmap) = self.font.rasterize(character, font_size);

        let glyph_image = Image {
            dimensions: (metrics.width as _, metrics.height as _),
            data: bitmap
                .iter()
                .flat_map(|a| vec![255, 255, 255, (*a)])
                .collect::<Vec<u8>>(),
            format: TextureFormat::Rgba8UnormSrgb,
            sampler: super::texture::TextureSamplerType::Nearest,
        };

        (metrics, glyph_image)
    }
}

impl Renderer {
    pub fn add_font(&mut self, font_data: &[u8]) -> FontResult<ArenaId<Font>> {
        let font = Font::try_from_bytes(font_data)?;

        Ok(self.fonts.insert(font))
    }

    pub fn add_font_as_default(&mut self, font_data: &[u8]) -> FontResult<ArenaId<Font>> {
        let font = Font::try_from_bytes(font_data)?;
        let default_id = ArenaId::first();
        *self
            .fonts
            .get_mut(default_id)
            .expect("Missing default font") = font;

        Ok(default_id)
    }
}
