use std::collections::HashMap;

use fontdue::Metrics;
use glam::Vec2;

use wgpu::TextureFormat;

use super::texture::{self, Image};

use super::dynamic_texture_atlas_builder::DynamicTextureAtlasBuilder;

pub(crate) struct FontAtlas {
    pub dynamic_texture_atlas_builder: DynamicTextureAtlasBuilder,
    pub glyph_atlas_info: HashMap<char, (usize, Metrics)>,
}

impl FontAtlas {
    pub fn new(size: Vec2) -> FontAtlas {
        let temp_image_data = Image {
            data: vec![
                0;
                (size.x * size.y) as usize
                    * TextureFormat::Rgba8UnormSrgb.block_size(None).unwrap() as usize
            ],
            format: TextureFormat::Rgba8UnormSrgb,
            dimensions: (size.x as u32, size.y as u32),
            sampler: texture::TextureSamplerType::Nearest,
        };

        let dynamic_texture_atlas_builder =
            DynamicTextureAtlasBuilder::new(size, 1, temp_image_data);

        Self {
            glyph_atlas_info: HashMap::default(),
            dynamic_texture_atlas_builder,
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
        glyph: char,
        image: &Image,
        glyph_metrics: Metrics,
    ) -> Option<Image> {
        if let Some((index, new_image)) = self.dynamic_texture_atlas_builder.add_texture(image) {
            self.glyph_atlas_info.insert(glyph, (index, glyph_metrics));
            return Some(new_image);
        }

        None
    }
}
