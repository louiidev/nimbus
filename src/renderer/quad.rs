use glam::{Quat, Vec2, Vec3};

use crate::{components::color::Color, material::Material, Transform};

#[derive(Default)]
pub struct Quad {
    pub transform: Transform,
    pub color: Color,
    pub material: Material,
}

impl Quad {
    pub fn from_size(size: Vec2, position: Vec3) -> Self {
        Self {
            transform: Transform {
                position,
                rotation: Quat::IDENTITY,
                scale: Vec3::new(size.x, 1.0, size.y),
            },
            color: Color::WHITE,
            material: Material::default(),
        }
    }

    pub fn new(transform: Transform, color: Color) -> Self {
        Self {
            transform,
            color,
            material: Material::default(),
        }
    }
}
