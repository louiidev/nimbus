use glam::Vec2;

use crate::arena::ArenaId;

use super::{rect::Rect, texture::Texture};

#[derive(Clone, Debug)]
pub struct TextureAtlas {
    /// The specific areas of the atlas where each texture can be found
    pub textures: Vec<Rect>,
    pub texture_handle: ArenaId<Texture>,
    pub size: Vec2,
    pub tile_size: Vec2,
}

impl Default for TextureAtlas {
    fn default() -> Self {
        Self {
            textures: Default::default(),
            texture_handle: ArenaId::first(),
            size: Default::default(),
            tile_size: Default::default(),
        }
    }
}

impl TextureAtlas {
    pub fn new_empty(size: Vec2) -> Self {
        Self {
            size,
            textures: Vec::new(),
            tile_size: Vec2::default(),
            texture_handle: ArenaId::first(),
        }
    }

    pub fn new(
        texture_handle: ArenaId<Texture>,
        tile_size: Vec2,
        columns: usize,
        rows: usize,
    ) -> Self {
        TextureAtlas::new_padding_offset(texture_handle, tile_size, columns, rows, None, None)
    }

    pub fn new_padding_offset(
        texture_handle: ArenaId<Texture>,
        tile_size: Vec2,
        columns: usize,
        rows: usize,
        padding: Option<Vec2>,
        offset: Option<Vec2>,
    ) -> Self {
        let padding = padding.unwrap_or_default();
        let offset = offset.unwrap_or_default();
        let mut current_padding = Vec2::ZERO;

        let mut textures = Vec::new();

        for y in 0..rows {
            if y > 0 {
                current_padding.y = padding.y;
            }
            for x in 0..columns {
                if x > 0 {
                    current_padding.x = padding.x;
                }

                let cell = Vec2::new(x as f32, y as f32);
                let rect_min = (tile_size + current_padding) * cell + offset;

                let sprite = Rect {
                    min: rect_min,
                    max: rect_min + tile_size,
                    ..Default::default()
                };

                textures.push(sprite);
            }
        }

        let grid_size = Vec2::new(columns as f32, rows as f32);

        TextureAtlas {
            texture_handle,
            textures,
            size: ((tile_size + current_padding) * grid_size) - current_padding,
            tile_size,
        }
    }

    pub(crate) fn add_texture(&mut self, rect: Rect) -> usize {
        self.textures.push(rect);
        self.textures.len() - 1
    }
}
