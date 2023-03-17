use fontdue::layout::{HorizontalAlign, VerticalAlign};
use glam::Vec2;

use crate::{
    color::Color,
    components::text::{Text, TextAlignment, TextTheme},
    rect::Rect,
    renderer::{RenderBatchMeta, QUAD_INDICES, QUAD_UVS, QUAD_VERTEX_POSITIONS},
    transform::Transform,
    DEFAULT_TEXTURE_ID,
};

use super::{id::Id, widget::WidgetResponse, UiHandler, UiVertex};

#[derive(Default)]
pub struct Button<'a> {
    pub text: &'a str,
    pub theme: ButtonTheme,
    pub active: bool,
    pub hover: bool,
    pub position: Vec2,
    pub id: Id,
}

impl<'a> Button<'a> {
    pub fn ui(&mut self, ui: &mut UiHandler) -> super::widget::WidgetResponse {
        self.id = UiHandler::generate_id(self.text, "button");

        self.position = ui.get_next_widget_position();
        let size = Vec2::new(250., 100.);

        let rect = Rect::new(
            self.position.x,
            self.position.y,
            self.position.x + size.x,
            self.position.y + size.y,
        );

        // Need to cache last active to see if we should apply click
        let last_frame_active_id = ui.active_id;

        ui.check_widget_interactions(self.id, size, self.position);

        self.hover = ui.hover_id == Some(self.id);

        let clicked =
            last_frame_active_id == Some(self.id) && ui.active_id != Some(self.id) && self.hover;

        self.active = ui.active_id == Some(self.id);

        let last_index = ui.current_layout.len() - 1;

        let layout = ui
            .current_layout
            .get_mut(last_index)
            .expect("Button needs to be inside layout to render");

        layout.ui_meta.push(self.get_render_meta());

        layout.push_widget(size);

        ui.text(
            Text {
                alignment: TextAlignment {
                    vertical: VerticalAlign::Middle,
                    horizontal: HorizontalAlign::Center,
                },
                value: self.text.to_string(),
                theme: TextTheme {
                    font_size: self.theme.font_size,
                    color: self.theme.text_color,
                },
                ..Default::default()
            },
            rect.min,
            Some(rect.max),
        );

        WidgetResponse { clicked }
    }

    fn get_render_meta(&self) -> RenderBatchMeta<UiVertex> {
        let size = Vec2::new(250., 100.);

        let transform = Transform::from_xyz(self.position.x, self.position.y, 1.0);

        let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
            (transform // offset the center point so it renders top left
                .transform_point(((quad_pos - Vec2::new(-0.5, -0.5) ) * size).extend(1.)))
            .into()
        });

        let background_color = if self.active {
            Color::rgb(
                self.theme.background_color.red - 0.2,
                self.theme.background_color.green - 0.2,
                self.theme.background_color.blue - 0.2,
            )
        } else if self.hover {
            Color::rgb(
                self.theme.background_color.red - 0.1,
                self.theme.background_color.green - 0.1,
                self.theme.background_color.blue - 0.1,
            )
        } else {
            self.theme.background_color
        };

        let mut vertices = Vec::new();

        for i in 0..QUAD_VERTEX_POSITIONS.len() {
            vertices.push(UiVertex {
                position: positions[i],
                tex_coords: QUAD_UVS[i].into(),
                color: background_color.as_rgba_f32(),
            });
        }

        RenderBatchMeta {
            texture_id: DEFAULT_TEXTURE_ID,
            vertices,
            indices: QUAD_INDICES.to_vec(),
        }
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
    pub text_color: Color,
    pub background_color: Color,
    pub font_id: Option<uuid::Uuid>,
    pub font_size: f32,
}

impl Default for ButtonTheme {
    fn default() -> Self {
        Self {
            text_color: Color::WHITE,
            background_color: Color::BLUE,
            font_id: None,
            font_size: 32.,
        }
    }
}
