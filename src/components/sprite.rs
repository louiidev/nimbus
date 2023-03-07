use crate::{
    color::Color,
    rect::Rect,
    transform::{GlobalTransform, Transform},
    DEFAULT_TEXTURE_ID,
};

use bevy_ecs::prelude::{Bundle, Component};
use glam::Vec2;

use super::collider::DebugCollider;

#[derive(Bundle, Default, Debug)]
pub struct SpriteBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    #[cfg(debug_assertions)]
    pub debug_collider: DebugCollider,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Sprite {
    pub texture_id: uuid::Uuid,
    pub anchor: Anchor,
    pub color: Color,
    pub texture_rect: Option<Rect>,
    pub custom_size: Option<Vec2>,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            texture_id: DEFAULT_TEXTURE_ID,
            anchor: Anchor::default(),
            color: Color::WHITE,
            texture_rect: None,
            custom_size: None,
        }
    }
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
