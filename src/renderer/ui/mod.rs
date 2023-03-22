use glam::Vec2;

use self::layout::{Layout, LayoutTheme};

use super::mesh2d::Mesh2d;

pub mod layout;

#[derive(Default)]
pub struct Ui {
    canvas_size: Vec2,
    current_layout: Vec<Layout>,
    pub(crate) render_meta: Vec<Mesh2d>,
}

impl Ui {
    pub fn new(canvas_size: Vec2) -> Self {
        Self {
            canvas_size,
            current_layout: Vec::default(),
            render_meta: Vec::default(),
        }
    }
    pub fn get_available_h(&self) -> f32 {
        let current_layout_height = self
            .current_layout
            .last()
            .as_ref()
            .map(|layout| layout.height())
            .unwrap_or(0.);

        self.canvas_size.x - current_layout_height
    }

    pub fn get_available_w(&self) -> f32 {
        let current_layout_width = self
            .current_layout
            .last()
            .as_ref()
            .map(|layout| layout.width())
            .unwrap_or(0.);

        self.canvas_size.x - current_layout_width
    }

    pub fn panel(&mut self, callback: impl FnOnce(&mut Self)) {
        let size = Vec2::new(self.get_available_w(), self.get_available_h());
        let layout = Layout {
            size,
            children: Vec::default(),
            layout_theme: Some(LayoutTheme::default()),
        };

        self.push_layout(layout);
        callback(self);
        self.pop_layout();
    }

    fn push_layout(&mut self, layout: Layout) {
        self.current_layout.push(layout);
    }

    fn pop_layout(&mut self) {
        let mut layout = self
            .current_layout
            .pop()
            .expect("Missing layout when popping");

        self.render_meta.append(&mut layout.get_render_meta());
    }

    pub(crate) fn resize(&mut self, canvas_size: Vec2) {
        self.canvas_size = canvas_size;
    }

    pub(crate) fn reset(&mut self) {
        self.current_layout = Vec::default();
        self.render_meta = Vec::default();
    }
}
