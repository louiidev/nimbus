use glam::Vec2;

use crate::{color::Color, renderer::RenderBatchMeta};

use super::UiVertex;

pub struct LayoutTheme {
    pub background_color: Color,
}

#[derive(Default)]
pub enum LayoutType {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Default)]
pub struct Layout {
    layout_type: LayoutType,
    pub(crate) position: Vec2,
    pub(crate) padding: f32,
    pub current_size: Vec2,
    pub(crate) ui_meta: Vec<RenderBatchMeta<UiVertex>>,
    pub layout_theme: Option<LayoutTheme>,
}

impl Layout {
    pub fn get_next_position(&self) -> Vec2 {
        match self.layout_type {
            LayoutType::Horizontal => {
                Vec2::new(self.current_size.x + self.position.x, self.position.y)
            }
            LayoutType::Vertical => {
                Vec2::new(self.position.x, self.current_size.y + self.position.y)
            }
        }
    }

    pub fn push_widget(&mut self, size: Vec2) {
        self.current_size = match self.layout_type {
            LayoutType::Horizontal => {
                let new_y = if size.y > self.current_size.y {
                    size.y
                } else {
                    self.current_size.y
                };

                Vec2::new(self.current_size.x + size.x + self.padding, new_y)
            }
            LayoutType::Vertical => {
                let new_x = if size.x > self.current_size.x {
                    size.x
                } else {
                    self.current_size.x
                };

                Vec2::new(new_x, self.current_size.y + size.y + self.padding)
            }
        }
    }
}
