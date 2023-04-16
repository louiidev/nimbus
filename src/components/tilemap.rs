use std::collections::HashMap;

use glam::{UVec2, Vec2, Vec3};

use crate::renderer::Renderer;

use super::{rect::Rect, sprite::Anchor, texture_atlas::TextureAtlas, transform::Transform};

type Tiles = Vec<usize>;

pub const NULL_TILE: usize = 0;

#[derive(PartialEq, Eq, Hash)]
pub enum TilemapLayer {
    Foreground,
    Stage,
    Background,
}

pub struct Tilemap {
    pub texture_atlas: TextureAtlas,
    pub layers: HashMap<TilemapLayer, Vec<usize>>,
    pub dimensions: UVec2, // in tiles so 8x8 or whatever
    pub transform: Transform,
    pub collisions: Vec<Rect>,
}

impl Tilemap {
    pub fn new(
        texture_atlas: TextureAtlas,
        layers: HashMap<TilemapLayer, Vec<usize>>,
        dimensions: UVec2,
        transform: Transform,
    ) -> Self {
        for layer in layers.values() {
            debug_assert_eq!(layer.len(), (dimensions.x * dimensions.y) as usize);
        }

        let mut collisions = Vec::default();

        let tile_size = texture_atlas.tile_size;
        let offset = ((tile_size * dimensions.as_vec2()).extend(0.) * transform.scale) / 2.;

        let stage_layer = layers.get(&TilemapLayer::Stage).unwrap();

        for x in 0..dimensions.x {
            for y in (0..dimensions.y).rev() {
                let tile = stage_layer[(y * dimensions.x + x) as usize];

                if tile == NULL_TILE {
                    continue;
                }

                let position = Vec2::new(
                    x as f32 * tile_size.x * transform.scale.x,
                    -(y as f32) * tile_size.y * transform.scale.y,
                );

                let origin =
                    transform.translation.truncate() + position - Vec2::new(offset.x, -offset.y);

                let size = transform.scale.truncate() * tile_size;

                collisions.push(Rect::from_center_size(
                    origin - (size * Anchor::TopLeft.as_vec()),
                    size,
                ))
            }
        }

        Self {
            texture_atlas,
            layers,
            dimensions,
            transform,
            collisions,
        }
    }

    fn draw_layer(
        &self,
        renderer: &mut Renderer,
        layer: &Tiles,
        tile_size: Vec2,
        offset: Vec3,
        layer_z: f32,
    ) {
        for x in 0..self.dimensions.x {
            for y in (0..self.dimensions.y).rev() {
                let tile = layer[(y * self.dimensions.x + x) as usize];
                if tile == NULL_TILE {
                    continue;
                    // continue;
                }

                let texture_rect = self.texture_atlas.textures.get(tile - 1).copied(); // in the tilemap we're gonna offset tiles by 1 so that 0 can be null
                let position = Vec3::new(
                    x as f32 * tile_size.x * self.transform.scale.x,
                    -(y as f32) * tile_size.y * self.transform.scale.y,
                    layer_z,
                );
                renderer.draw_sprite(
                    &super::sprite::Sprite {
                        texture_id: self.texture_atlas.texture_id,
                        texture_rect,
                        anchor: Anchor::TopLeft,
                        ..Default::default()
                    },
                    &Transform {
                        translation: self.transform.translation + position
                            - Vec3::new(offset.x, -offset.y, 0.),
                        ..self.transform
                    },
                );
            }
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        let tile_size = self.texture_atlas.tile_size;
        let offset =
            ((tile_size * self.dimensions.as_vec2()).extend(0.) * self.transform.scale) / 2.;

        if let Some(layer) = self.layers.get(&TilemapLayer::Background) {
            self.draw_layer(renderer, layer, tile_size, offset, -0.1);
        }

        if let Some(layer) = self.layers.get(&TilemapLayer::Stage) {
            self.draw_layer(renderer, layer, tile_size, offset, 0.0);
        }

        if let Some(layer) = self.layers.get(&TilemapLayer::Foreground) {
            self.draw_layer(renderer, layer, tile_size, offset, 1.0);
        }
    }
}
