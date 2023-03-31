use fontdue::layout::{HorizontalAlign, VerticalAlign};

use crate::arena::ArenaId;

use super::color::Color;

#[derive(Clone)]
pub struct Text {
    pub alignment: TextAlignment,
    pub value: String,
    pub theme: TextTheme,
    pub font_id: ArenaId,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            alignment: TextAlignment::default(),
            value: String::default(),
            theme: TextTheme::default(),
            font_id: ArenaId::first(),
        }
    }
}

#[derive(Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct TextTheme {
    pub font_size: f32,
    pub color: Color,
}

impl Default for TextTheme {
    fn default() -> Self {
        Self {
            font_size: 24f32,
            color: Color::WHITE,
        }
    }
}
