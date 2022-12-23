use std::collections::HashMap;

use glam::Vec2;

use crate::{components::sprite::Sprite, rect::Rect};

pub fn from_texture_atlas(
    texture_id: uuid::Uuid,
    tile_size: Vec2,
    columns: usize,
    rows: usize,
    padding: Option<Vec2>,
    offset: Option<Vec2>,
) -> HashMap<(usize, usize), Sprite> {
    let padding = padding.unwrap_or_default();
    let offset = offset.unwrap_or_default();
    let mut current_padding = Vec2::ZERO;

    let mut sprites = HashMap::new();

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

            let sprite = Sprite {
                texture_rect: Some(Rect {
                    min: rect_min,
                    max: rect_min + tile_size,
                }),
                texture_id,
                ..Default::default()
            };

            sprites.insert((x, y), sprite);
        }
    }

    let _grid_size = Vec2::new(columns as f32, rows as f32);

    sprites
}
