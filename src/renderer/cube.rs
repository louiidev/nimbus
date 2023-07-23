use glam::Vec3;

use crate::{components::color::Color, pipeline::Pipeline, ArenaId, Texture};

#[derive(Debug, Default)]
pub struct Cube {
    pub size: Vec3,
    pub color: Color,
    pub texture: ArenaId<Texture>,
    pub material: ArenaId<Pipeline>,
}

impl Cube {
    pub fn new(size: Vec3, color: Color, material: ArenaId<Pipeline>) -> Self {
        Cube {
            size,
            color,
            texture: ArenaId::first(),
            material,
        }
    }
}
