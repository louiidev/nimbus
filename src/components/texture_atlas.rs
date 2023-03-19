use glam::{IVec2, Vec2};

use crate::areana::ArenaId;

use super::rect::Rect;

#[derive(Clone)]
pub struct TextureAtlas {
    texture_id: ArenaId,
    /// The specific areas of the atlas where each texture can be found
    pub textures: Vec<Rect>,
    pub size: Vec2,
}

impl TextureAtlas {
    pub fn new_empty(texture_id: ArenaId, size: Vec2) -> Self {
        Self {
            texture_id,
            size,
            textures: Vec::new(),
        }
    }

    pub fn from_texture_atlas(
        texture_id: ArenaId,
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
                };

                textures.push(sprite);
            }
        }

        let grid_size = Vec2::new(columns as f32, rows as f32);

        TextureAtlas {
            texture_id,
            textures,
            size: ((tile_size + current_padding) * grid_size) - current_padding,
        }
    }

    fn add_texture(&mut self, rect: Rect) -> usize {
        self.textures.push(rect);
        self.textures.len() - 1
    }
}

fn to_rect(rectangle: guillotiere::Rectangle) -> Rect {
    Rect {
        min: IVec2::new(rectangle.min.x, rectangle.min.y).as_vec2(),
        max: IVec2::new(rectangle.max.x, rectangle.max.y).as_vec2(),
    }
}

fn to_size2(vec2: Vec2) -> guillotiere::Size {
    guillotiere::Size::new(vec2.x as i32, vec2.y as i32)
}
