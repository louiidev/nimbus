use ab_glyph::FontArc;
use ab_glyph::{FontVec, InvalidFont, OutlinedGlyph};
use glam::Vec2;
use wgpu::{Extent3d, TextureDimension, TextureFormat};

use crate::transform::Transform;
use crate::{internal_image::Image, utils::float_ord::FloatOrd};

pub type FontSizeKey = FloatOrd;

#[derive(Debug, Clone)]
pub struct GlyphAtlasInfo {
    pub texture_atlas_id: uuid::Uuid,
    pub glyph_index: usize,
}

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub position: Vec2,
    pub size: Vec2,
    pub atlas_info: GlyphAtlasInfo,
    pub section_index: usize,
    pub byte_index: usize,
    pub glyph_transform: Transform,
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
