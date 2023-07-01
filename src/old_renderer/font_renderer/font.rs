use crate::components::dynamic_texture_atlas_builder::TempImageData;
use crate::utils::float_ord::FloatOrd;
use crate::{arena::ArenaId, components::rect::Rect};
use core::hash::{Hash, Hasher};
use fontdue::{Font as ExternalFont, FontResult, Metrics};
use glam::{IVec2, Vec2};
use wgpu::TextureFormat;

pub type FontSizeKey = FloatOrd;

#[derive(Debug, Clone)]
pub struct GlyphAtlasInfo {
    pub texture_rect: Rect,
    pub metrics: Metrics,
    pub texture_id: ArenaId,
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
}

impl Hash for Font {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font.hash(state);
    }
}

impl Font {
    pub fn try_from_bytes(font_data: &[u8]) -> FontResult<Self> {
        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())?;
        Ok(Font { font })
    }

    pub(crate) fn rasterize(&self, character: char, font_size: f32) -> (Metrics, TempImageData) {
        let (metrics, bitmap) = self.font.rasterize(character, font_size);

        let glyph_image = TempImageData {
            size: IVec2::new(metrics.width as _, metrics.height as _),
            data: bitmap
                .iter()
                .flat_map(|a| vec![255, 255, 255, (*a)])
                .collect::<Vec<u8>>(),
            format: TextureFormat::Rgba8UnormSrgb,
        };

        (metrics, glyph_image)
    }
}
