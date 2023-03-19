use bevy_ecs::prelude::{Bundle, Component};
use glam::{Vec2, Vec3};

use crate::{
    color::Color,
    transform::{GlobalTransform, Transform},
};

use super::sprite::Anchor;

#[derive(Bundle, Default, Debug)]
pub struct ShapeBundle {
    pub shape: Shape,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Debug, Default, Component)]
pub struct Shape {
    pub(crate) vertices: Vec<Vec3>,
    pub(crate) indicies: Vec<u16>,
    pub anchor: Anchor,
    pub color: Color,
    pub(crate) tex_coords: Vec<Vec2>,
}

impl Shape {
    pub fn hexagon(color: Color) -> Self {
        Shape {
            vertices: vec![
                Vec3::new(0., 0., 0.),       // Center
                Vec3::new(0., 0.5, 0.0),     // Top
                Vec3::new(0.5, 0.25, 0.0),   // Top Right
                Vec3::new(0.5, -0.25, 0.0),  // Bottom Right
                Vec3::new(0., -0.5, 0.0),    // Bottom
                Vec3::new(-0.5, -0.25, 0.0), // Bottom Left
                Vec3::new(-0.5, 0.25, 0.0),  // Top Left
            ],
            indicies: vec![
                0, 1, 2, // Top Right
                0, 2, 3, // Right
                0, 3, 4, // Bottom Right,
                0, 5, 6, // Bottom Left
                0, 6, 1, // Left
            ],
            color,
            anchor: Anchor::Center,
            tex_coords: vec![
                Vec2::new(0.5, 0.5),
                Vec2::new(0.5, 0.0),
                Vec2::new(0.0, 0.25),
                Vec2::new(0.0, 0.75),
                Vec2::new(0.5, 0.0),
                Vec2::new(1.0, 0.75),
                Vec2::new(1.0, 0.25),
            ],
        }
    }
}
