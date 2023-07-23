use glam::Vec2;

use crate::{
    components::color::Color,
    mesh::{Mesh, MeshBuilder},
    Rect, Transform,
};

use super::Renderer;

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
    pub children: Vec<Mesh>,
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

    pub fn get_render_meta(&mut self) -> Vec<Mesh> {
        let mut meta = Vec::default();

        if let Some(theme) = &self.layout_theme {
            if theme.color != Color::NONE {
                meta.push(Rect::from_center_size(self.size, self.position).into_mesh(theme.color));
            }
        }

        meta.append(&mut self.children);

        meta
    }
}

impl Renderer {
    pub fn allocate_space(&mut self, size: Vec2) {
        let layout = self.current_layout.last_mut().unwrap();

        layout.allocated_space += match layout.layout_direction {
            LayoutDirection::Horizontal => size.x,
            LayoutDirection::Vertical => size.y,
        }
    }
    fn push_layout(&mut self, layout: Layout) {
        self.allocate_space(layout.size);
        self.current_layout.push(layout);
    }

    fn pop_layout(&mut self) {
        let mut layout = self
            .current_layout
            .pop()
            .expect("Missing layout when popping");

        self.ui_render_data.append(&mut layout.get_render_meta());
    }

    pub fn get_available_space(&self) -> Vec2 {
        let layout = self.current_layout.last().expect("Missing layout");
        match layout.layout_direction {
            LayoutDirection::Horizontal => {
                Vec2::new(layout.width() - layout.allocated_space, layout.height())
            }
            LayoutDirection::Vertical => {
                Vec2::new(layout.width(), layout.height() - layout.allocated_space)
            }
        }
    }

    pub fn get_next_available_position(&self) -> Vec2 {
        let layout = self.current_layout.last().expect("Missing layout");
        match layout.layout_direction {
            LayoutDirection::Horizontal => Vec2::new(
                layout.position.x + layout.allocated_space,
                layout.position.y,
            ),
            LayoutDirection::Vertical => Vec2::new(
                layout.position.x,
                layout.position.y + layout.allocated_space,
            ),
        }
    }

    pub fn panel(&mut self, callback: impl FnOnce(&mut Self)) {
        let available_space = self.get_available_space();
        let layout = Layout {
            size: available_space,
            layout_theme: Some(LayoutTheme::default()),
            position: self.get_next_available_position(),
            ..Default::default()
        };

        self.push_layout(layout);
        callback(self);
        self.pop_layout();
    }
}
