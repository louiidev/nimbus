use glam::Vec2;

use crate::{components::color::Color, renderer::mesh2d::Mesh2d};

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

pub struct Layout {
    pub size: Vec2,
    pub children: Vec<Mesh2d>,
    pub layout_theme: Option<LayoutTheme>,
}

impl Layout {
    pub fn width(&self) -> f32 {
        self.size.x
    }

    pub fn height(&self) -> f32 {
        self.size.y
    }

    pub fn get_render_meta(&mut self) -> Vec<Mesh2d> {
        let has_color = self
            .layout_theme
            .as_ref()
            .map(|theme| theme.color)
            .unwrap_or(Color::NONE)
            != Color::NONE;

        let mut meta = Vec::default();

        if has_color {
            todo!("Push colored square of size layout")
        }

        meta.append(&mut self.children);

        meta
    }
}
