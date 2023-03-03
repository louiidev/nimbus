use crate::rect::Rect;
use crate::{internal_image::Image, utils::float_ord::FloatOrd};
use fontdue::{Font, FontResult, Metrics};
use glam::Vec2;
use wgpu::{Extent3d, TextureDimension, TextureFormat};

pub type FontSizeKey = FloatOrd;

#[derive(Debug, Clone)]
pub struct GlyphAtlasInfo {
    pub texture_atlas_id: uuid::Uuid,
    pub glyph_index: usize,
    pub metrics: Metrics,
}

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub position: Vec2,
    pub rect: Rect,
    pub atlas_info: GlyphAtlasInfo,
}

pub struct FontData {
    pub font: Font,
}

impl FontData {
    pub fn try_from_bytes(font_data: &[u8]) -> FontResult<Self> {
        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())?;
        Ok(FontData { font })
    }

    pub fn rasterize(&self, character: char, font_size: f32) -> (Metrics, Image) {
        let (metrics, bitmap) = self.font.rasterize(character, font_size);
        dbg!(character, metrics);
        let image = Image::new(
            Extent3d {
                width: metrics.width as _,
                height: metrics.height as _,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            bitmap
                .iter()
                .flat_map(|a| vec![255, 255, 255, (*a)])
                .collect::<Vec<u8>>(),
            TextureFormat::Rgba8UnormSrgb,
        );

        (metrics, image)
    }
}
