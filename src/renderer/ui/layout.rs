use glam::Vec2;
use guillotiere::euclid::default;

use crate::{components::color::Color, renderer::mesh2d::Mesh2d};

#[derive(Debug)]
pub struct LayoutTheme {
    pub color: Color,
    pub padding: f32,
}

impl Default for LayoutTheme {
    fn default() -> Self {
        LayoutTheme {
            color: Color::DARK_GRAY,
            padding: 5f32,
        }
    }
}

#[derive(Debug, Default)]
pub enum LayoutDirection {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Default, Debug)]
pub struct Layout {
    pub size: Vec2,
    pub children: Vec<Mesh2d>,
    pub layout_theme: Option<LayoutTheme>,
    pub allocated_space: f32, // We use this and direction to determine how much space left to allocate in layout
    pub position: Vec2,
    pub layout_direction: LayoutDirection,
}

impl Layout {
    pub fn width(&self) -> f32 {
        self.size.x
    }

    pub fn height(&self) -> f32 {
        self.size.y
    }

    pub fn get_render_meta(&mut self) -> Vec<Mesh2d> {
        let mut meta = Vec::default();

        if let Some(theme) = &self.layout_theme {
            if theme.color != Color::NONE {
                meta.push(Mesh2d::rect(self.position, self.size, theme.color))
            }
        }

        meta.append(&mut self.children);

        meta
    }
}
