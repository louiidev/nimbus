use crate::color::Color;

use super::Widget;

#[derive(Default)]
pub struct Button<'a> {
    pub text: &'a str,
    pub theme: ButtonTheme,
}

impl<'a> Widget for Button<'a> {
    fn get_size(&self) -> glam::Vec2 {
        todo!()
    }

    fn get_render_meta(
        &self,
        position_to_render: glam::Vec2,
    ) -> crate::renderer::RenderBatchMeta<super::UiVertex> {
        todo!()
    }
}
pub struct ButtonState {
    pub(crate) clicked: bool,
}

impl ButtonState {
    pub fn is_clicked(&self) -> bool {
        self.clicked
    }
}

pub struct ButtonTheme {
    text_color: Color,
    background_color: Color,
    font_id: Option<uuid::Uuid>,
}

impl Default for ButtonTheme {
    fn default() -> Self {
        Self {
            text_color: Color::WHITE,
            background_color: Color::RED,
            font_id: None,
        }
    }
}
