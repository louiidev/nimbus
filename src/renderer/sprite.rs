use std::default;

use crate::{
    color::Color,
    rect::Rect,
    renderer::mesh::Mesh,
    transform::{GlobalTransform, Transform},
};

use bevy_ecs::prelude::{Bundle, Component};
use glam::Vec2;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Component)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Bundle, Default, Debug)]
pub struct SpriteBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Component, Clone, Copy, Default, Debug)]
pub struct Sprite {
    pub(crate) texture_id: uuid::Uuid,
    pub(crate) anchor: Anchor,
    pub color: Color,
    pub texture_rect: Option<Rect>,
    pub custom_size: Option<Vec2>,
}

impl Sprite {
    pub fn new(texture_id: uuid::Uuid) -> Self {
        Self {
            texture_id,
            anchor: Anchor::default(),
            color: Color::WHITE,
            texture_rect: None,
            custom_size: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Anchor {
    #[default]
    Center,
    BottomLeft,
    BottomCenter,
    BottomRight,
    CenterLeft,
    CenterRight,
    TopLeft,
    TopCenter,
    TopRight,
    /// Custom anchor point. Top left is `(-0.5, 0.5)`, center is `(0.0, 0.0)`. The value will
    /// be scaled with the sprite size.
    Custom(Vec2),
}

impl Anchor {
    pub fn as_vec(&self) -> Vec2 {
        match self {
            Anchor::Center => Vec2::ZERO,
            Anchor::BottomLeft => Vec2::new(-0.5, -0.5),
            Anchor::BottomCenter => Vec2::new(0.0, -0.5),
            Anchor::BottomRight => Vec2::new(0.5, -0.5),
            Anchor::CenterLeft => Vec2::new(-0.5, 0.0),
            Anchor::CenterRight => Vec2::new(0.5, 0.0),
            Anchor::TopLeft => Vec2::new(-0.5, 0.5),
            Anchor::TopCenter => Vec2::new(0.0, 0.5),
            Anchor::TopRight => Vec2::new(0.5, 0.5),
            Anchor::Custom(point) => *point,
        }
    }
}
