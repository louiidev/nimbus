use std::collections::HashMap;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::Metrics;
use glam::Vec2;

use wgpu::{Extent3d, TextureDimension, TextureFormat};

use crate::arena::{Arena, ArenaId};
use crate::components::dynamic_texture_atlas_builder::{DynamicTextureAtlasBuilder, TempImageData};

pub(crate) struct FontAtlas {
    pub dynamic_texture_atlas_builder: DynamicTextureAtlasBuilder,
    pub glyph_atlas_info: HashMap<char, (usize, Metrics)>,
}

impl FontAtlas {
    pub fn new(size: Vec2) -> FontAtlas {
        let temp_image_data = TempImageData {
            data: vec![
                0;
                (size.x * size.y) as usize
                    * TextureFormat::Rgba8UnormSrgb.describe().block_size as usize
            ],
            size: size.as_ivec2(),
            format: TextureFormat::Rgba8UnormSrgb,
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
        image: &TempImageData,
        glyph_metrics: Metrics,
    ) -> Option<TempImageData> {
        if let Some((index, new_image)) = self.dynamic_texture_atlas_builder.add_texture(image) {
            self.glyph_atlas_info.insert(glyph, (index, glyph_metrics));
            return Some(new_image);
        }

        None
    }
}
