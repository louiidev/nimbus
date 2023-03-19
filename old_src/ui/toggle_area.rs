use glam::Vec2;

use crate::{
    color::Color,
    components::text::{Text, TextAlignment, TextTheme},
    DEFAULT_FONT_ID, DEFAULT_TEXTURE_ID, TOGGLE_TRIANGLE_TEXTURE_ID,
};

use super::{id::Id, UiHandler};

// CONST styling, TODO: maybe load from a file in the future
const TOGGLE_INSET: f32 = 20f32;
const TOGGLE_PADDING: f32 = 15f32;
const TOGGLE_FONT_SIZE: f32 = 20f32;
const TOGGLE_TRIANGLE_SIZE: f32 = 12f32;

#[derive(Default)]
pub struct ToggleArea<'a> {
    pub heading: &'a str,
    pub theme: ToggleAreaTheme,
    pub hover: bool,
    pub id: Id,
    pub default_open: bool,
}

impl<'a> ToggleArea<'a> {
    pub fn new(heading: &'a str) -> Self {
        ToggleArea {
            heading,
            ..Default::default()
        }
    }

    pub fn ui(&mut self, ui: &mut UiHandler, callback: impl FnOnce(&mut UiHandler)) {
        let mut layout_position = ui.get_next_widget_position();

        ui.panel(layout_position, |ui| {
            self.id = UiHandler::generate_id(self.heading, "toggle_area");

            let _ = ui.toggle_states.try_insert(self.id, self.default_open);

            let triangle_size = Vec2::splat(TOGGLE_TRIANGLE_SIZE);
            let text_size = ui.measure_text(
                Text {
                    value: self.heading.into(),
                    alignment: TextAlignment {
                        vertical: fontdue::layout::VerticalAlign::Top,
                        horizontal: fontdue::layout::HorizontalAlign::Left,
                    },
                    theme: TextTheme {
                        font_size: TOGGLE_FONT_SIZE,
                        color: Color::WHITE,
                    },
                    font_id: DEFAULT_FONT_ID,
                },
                None,
            );

            // Need to cache last active to see if we should apply click
            let last_frame_active_id = ui.active_id;

            ui.check_widget_interactions(self.id, triangle_size + text_size, layout_position);

            self.hover = ui.hover_id == Some(self.id);

            let clicked = last_frame_active_id == Some(self.id)
                && ui.active_id != Some(self.id)
                && self.hover;

            let active = ui.active_id == Some(self.id);

            if clicked {
                let value = ui.toggle_states.get_mut(&self.id).unwrap();

                *value = !*value;
            }

            let opened = *ui.toggle_states.get(&self.id).unwrap();

            let rotation = if opened { 90f32 } else { 0f32 };

            ui.rect(
                triangle_size,
                TOGGLE_TRIANGLE_TEXTURE_ID,
                rotation,
                Some(Vec2::new(0., 5f32)),
            );
            layout_position.x += TOGGLE_INSET;

            ui.text(
                Text {
                    value: self.heading.into(),
                    theme: TextTheme {
                        font_size: TOGGLE_FONT_SIZE,
                        color: Color::WHITE,
                    },
                    ..Default::default()
                },
                layout_position,
                None,
            );

            let text_height = text_size.y;

            let layout_position = Vec2::new(
                layout_position.x + TOGGLE_INSET,
                layout_position.y + text_height,
            );

            if !opened {
                return;
            }

            ui.layout(layout_position, TOGGLE_PADDING, callback);
        })
    }
}

impl UiHandler {
    pub fn toggle_area(&mut self, mut toggle_area: ToggleArea, callback: impl FnOnce(&mut Self)) {
        toggle_area.ui(self, callback);
    }
}

pub struct ToggleAreaTheme {
    pub text_color: Color,
    pub background_color: Color,
    pub font_id: Option<ArenaId>,
    pub font_size: f32,
}

impl Default for ToggleAreaTheme {
    fn default() -> Self {
        Self {
            text_color: Color::WHITE,
            background_color: Color::BLUE,
            font_id: None,
            font_size: 32.,
        }
    }
}
