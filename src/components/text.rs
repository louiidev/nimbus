use bevy_ecs::prelude::{Bundle, Component};
use glyph_brush_layout::{HorizontalAlign, VerticalAlign};

use crate::{
    color::Color,
    transform::{GlobalTransform, Transform},
    DEFAULT_FONT_ID,
};

#[derive(Debug, Component)]
pub struct Text {
    pub alignment: TextAlignment,
    pub value: String,
    pub theme: TextTheme,
    pub font_id: uuid::Uuid,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            alignment: TextAlignment::default(),
            value: String::default(),
            theme: TextTheme::default(),
            font_id: DEFAULT_FONT_ID,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextAlignment {
    pub vertical: VerticalAlign,
    pub horizontal: HorizontalAlign,
}

impl Default for TextAlignment {
    fn default() -> Self {
        TextAlignment {
            vertical: VerticalAlign::Top,
            horizontal: HorizontalAlign::Left,
        }
    }
}

#[derive(Bundle, Default, Debug)]
pub struct TextBundle {
    pub text: Text,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Default, Debug)]
pub struct TextTheme {
    pub font_size: f32,
    pub color: Color,
}
