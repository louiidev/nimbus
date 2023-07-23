use glam::Vec3;

use crate::{components::color::Color, material::Material, ArenaId, Texture};

#[derive(Debug, Default)]
pub struct Cube {
    pub size: Vec3,
    pub color: Color,
    pub texture: ArenaId<Texture>,
    pub material: Material,
}

impl Cube {
    pub fn new(size: Vec3, color: Color, material: Material) -> Self {
        Cube {
            size,
            color,
            texture: ArenaId::first(),
            material,
        }
    }
}
