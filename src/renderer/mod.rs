use glam::{Vec2, Vec3};
use render_buddy::{
    line::LineMeshBuilder, rect::Rect, sprite::Sprite, text::Text, transform::Transform,
    RenderBuddy, SortingAxis,
};

use crate::components::{color::Color, line::Line2D};

pub struct Renderer {
    pub(crate) render_buddy: RenderBuddy,
}
impl Renderer {
    pub fn set_sorting_axis_2d(&mut self, sorting_axis: SortingAxis) {
        self.render_buddy.set_sorting_axis(sorting_axis);
    }

    pub fn draw_rect(&mut self, rect: &Rect, position: Vec3) {
        self.render_buddy.push(*rect, position);
    }

    pub fn draw_sprite_basic(&mut self, sprite: &Sprite, position: Vec3) {
        self.render_buddy.push(*sprite, position)
    }

    pub fn draw_sprite(&mut self, sprite: Sprite, transform: Transform) {
        self.render_buddy.push_transform(sprite, transform)
    }

    pub fn draw_text(&mut self, text: Text, transform: Transform) {
        self.render_buddy.append_transform(text, transform);
    }

    pub fn draw_text_basic(&mut self, text: Text, position: Vec3) {
        self.render_buddy.append(text, position);
    }

    pub fn draw_line(&mut self, line: Line2D, color: Color) {
        self.render_buddy.push_mesh(LineMeshBuilder::new().line(
            line.0.extend(0.),
            line.1.extend(0.),
            color.into(),
        ));
    }

    pub fn draw_line_rect(&mut self, rect: Rect, color: Color) {
        self.render_buddy
            .push_mesh(LineMeshBuilder::new().rect(rect, color.into()));
    }

    pub fn get_viewport_size(&self) -> Vec2 {
        let (width, height) = self.render_buddy.get_viewport_size();
        Vec2::new(width as f32, height as f32)
    }

    pub fn measure_text(&mut self, text: &Text) -> Vec2 {
        self.render_buddy.measure_text(&text)
    }
}
