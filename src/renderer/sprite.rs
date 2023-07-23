use glam::Vec2;

use crate::{arena::ArenaId, material::Material};

use super::{rect::Rect, texture::Texture};

#[derive(Clone, Debug)]
pub struct Sprite {
    pub material: Material,
    pub anchor: Anchor,
    pub color: [f32; 4],
    pub texture_rect: Option<Rect>,
    pub custom_size: Option<Vec2>,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Default for Sprite {
    fn default() -> Self {
        Sprite {
            material: Material::default(),
            anchor: Anchor::default(),
            color: [1., 1., 1., 1.],
            texture_rect: None,
            custom_size: None,
            flip_x: false,
            flip_y: false,
        }
    }
}
impl Sprite {
    pub fn new(texture: ArenaId<Texture>) -> Self {
        Sprite {
            material: Material::default().with_texture(texture),
            ..Default::default()
        }
    }

    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_flip_x(mut self, flip_x: bool) -> Self {
        self.flip_x = flip_x;
        self
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
