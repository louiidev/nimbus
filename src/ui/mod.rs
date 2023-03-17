use std::{
    f32::{MAX, MIN},
    string,
};

use bevy_ecs::system::Resource;
use glam::{Mat4, Vec2, Vec3, Vec4Swizzles};
use hashbrown::HashMap;
use winit::event::MouseButton;

use crate::{
    color::Color,
    components::text::{Text, TextAlignment, TextTheme},
    font::FontData,
    font_atlas::FontAtlasSet,
    internal_image::Image,
    rect::Rect,
    renderer::{RenderBatchMeta, QUAD_INDICES, QUAD_UVS, QUAD_VERTEX_POSITIONS},
    resources::{inputs::InputController, utils::Assets},
    texture_atlas::TextureAtlas,
    transform::{GlobalTransform, Transform},
    utils::collision,
    DEFAULT_FONT_ID, DEFAULT_TEXTURE_ID,
};

use self::{
    button::Button,
    id::Id,
    layout::{Layout, LayoutTheme},
    toggle_area::ToggleArea,
    widget::WidgetResponse,
};

pub mod button;
pub mod id;
pub mod layout;
pub mod toggle_area;
pub mod widget;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

impl UiVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<UiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 3]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub enum Constraint {
    Responsive,
    Pixel(f32),
}

pub struct UiConstraint {
    width: Constraint,
    height: Constraint,
}

#[derive(Resource)]
pub struct UiHandler {
    pub queued_layouts: Vec<RenderBatchMeta<UiVertex>>,
    current_layout: Vec<Layout>,
    pub texture_atlases: Assets<TextureAtlas>,
    pub texture_atlases_images: Assets<Image>,
    pub input_controller: InputController,
    pub active_id: Option<Id>,
    pub hover_id: Option<Id>,
    pub font_atlas: FontAtlasSet,
    pub(crate) fonts: Assets<FontData>,
    pub(crate) container_size: Vec2,
    pub(crate) toggle_states: HashMap<Id, bool>,
}

impl Default for UiHandler {
    fn default() -> Self {
        UiHandler {
            queued_layouts: Vec::default(),
            current_layout: Vec::default(),
            texture_atlases: Assets::new(),
            input_controller: InputController::default(),
            active_id: None,
            hover_id: None,
            font_atlas: FontAtlasSet::default(),
            fonts: Assets::new(),
            texture_atlases_images: Assets::default(),
            container_size: Vec2::default(),
            toggle_states: HashMap::default(),
        }
    }
}

impl UiHandler {
    pub fn new(window_size: Vec2) -> Self {
        // Prepare glyph_brush

        UiHandler {
            queued_layouts: Vec::new(),
            current_layout: Vec::new(),
            texture_atlases_images: Assets::new(),
            texture_atlases: Assets::new(),
            input_controller: InputController::default(),
            active_id: None,
            hover_id: None,
            font_atlas: FontAtlasSet::default(),
            fonts: Assets::new(),
            container_size: window_size,
            toggle_states: HashMap::default(),
        }
    }

    pub fn check_widget_interactions(&mut self, id: Id, size: Vec2, position: Vec2) {
        if self.hover_id == Some(id) || self.hover_id.is_none() {
            let is_hovering = self.check_hover(size, position);
            if !is_hovering {
                self.hover_id = None;
                self.active_id = None;
                return;
            }

            self.hover_id = Some(id);

            self.active_id = if self
                .input_controller
                .mouse_button_inputs
                .pressed(MouseButton::Left)
            {
                Some(id)
            } else {
                None
            }
        }
    }

    pub fn generate_id(key: &str, widget_type: &str) -> Id {
        Id::new(format!("Widget_{}_key_{}", widget_type, key))
    }

    pub fn get_current_layout_mut(&mut self) -> &mut Layout {
        let last_index = self.current_layout.len() - 1;
        self.current_layout
            .get_mut(last_index)
            .expect("generate_id needs to be called inside a layout to work")
    }

    pub fn get_current_layout(&self) -> &Layout {
        let last_index = self.current_layout.len() - 1;
        self.current_layout
            .get(last_index)
            .expect("generate_id needs to be called inside a layout to work")
    }

    pub fn get_next_widget_position(&self) -> Vec2 {
        let last_index = self.current_layout.len() - 1;

        let layout = self
            .current_layout
            .get(last_index)
            .expect("Button needs to be inside layout to render");

        layout.get_next_position()
    }

    pub fn check_hover(&mut self, size: Vec2, position: Vec2) -> bool {
        collision::rect_contains_point(size, position, self.input_controller.screen_mouse_position)
    }

    pub fn check_active(&mut self, id: Id) -> bool {
        Some(id) == self.hover_id
            && self
                .input_controller
                .mouse_button_inputs
                .pressed(MouseButton::Left)
    }

    pub fn button(&mut self, mut button: Button) -> WidgetResponse {
        button.ui(self)
    }

    pub fn triangle(&mut self) {
        let layout = self.get_current_layout_mut();

        let layout_position = layout.position;

        let mut vertices = Vec::new();

        let transform = Transform::from_xyz(layout_position.x, layout_position.y, 1.0);

        pub const TRI_VERTEX_POSITIONS: [Vec2; 3] = [
            Vec2::new(-0.5, -0.5),
            Vec2::new(0.5, 0.0),
            Vec2::new(-0.5, 0.5),
        ];

        let size = 15f32;

        let positions: [[f32; 3]; 3] = TRI_VERTEX_POSITIONS.map(|tri_pos| {
            (transform // offset the center point so it renders top left
                .transform_point(((tri_pos - Vec2::new(-0.5, -0.5)) * size).extend(1.)))
            .into()
        });

        for i in 0..TRI_VERTEX_POSITIONS.len() {
            vertices.push(UiVertex {
                position: positions[i],
                tex_coords: QUAD_UVS[i].into(),
                color: Color::WHITE.into(),
            });
        }

        let meta = RenderBatchMeta {
            texture_id: DEFAULT_TEXTURE_ID,
            vertices,
            indices: [0, 1, 2].to_vec(),
        };

        layout.ui_meta.push(meta);
    }

    pub fn rect(
        &mut self,
        size: Vec2,
        texture_id: uuid::Uuid,
        rotation_angle: f32,
        offset: Option<Vec2>,
    ) {
        let layout_position = self.get_next_widget_position() + offset.unwrap_or(Vec2::ZERO);
        let mut transform = Transform::from_xyz(
            layout_position.x + size.x / 2f32,
            layout_position.y + size.y / 2f32,
            1.0,
        );
        transform.rotate_z(rotation_angle.to_radians());
        let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
            (transform // offset the center point so it renders top left
                // TODO: make this a constant instead of doing the calculation every frame
                .transform_point(((quad_pos) * size).extend(1.)))
            .into()
        });

        let mut vertices = Vec::new();

        for i in 0..QUAD_VERTEX_POSITIONS.len() {
            vertices.push(UiVertex {
                position: positions[i],
                tex_coords: QUAD_UVS[i].into(),
                color: Color::ORANGE_RED.into(),
            });
        }

        let meta = RenderBatchMeta {
            texture_id,
            vertices,
            indices: QUAD_INDICES.to_vec(),
        };

        let layout = self.get_current_layout_mut();
        layout.push_widget(size);
        layout.ui_meta.push(meta);
    }

    pub fn measure_text(&mut self, text: Text, max_bounds: Option<Vec2>) -> Vec2 {
        let font = self.fonts.get(&DEFAULT_FONT_ID).unwrap();
        let rect = Rect {
            min: Vec2::ZERO,
            max: max_bounds.unwrap_or(self.container_size),
        };
        let text_glyphs = self.font_atlas.queue_text(
            font,
            &text,
            &rect,
            &mut self.texture_atlases,
            &mut self.texture_atlases_images,
            fontdue::layout::CoordinateSystem::PositiveYDown,
        );

        let mut size = Vec2::default();

        for text_glyph in text_glyphs {
            let glyph_size = text_glyph.rect.size();
            // Get top left glyph pos
            let glyph_position = text_glyph.position - -Vec2::new(-0.5, -0.5);

            let x_distance = glyph_position.x - size.x;

            let rect = Rect {
                min: glyph_position,
                max: glyph_position + glyph_size,
            };

            let actual_glyph_size = rect.size();

            size.y = size.y.max(actual_glyph_size.y);
            size.x += actual_glyph_size.x + x_distance;
        }

        size
    }

    pub fn panel_with_padding(
        &mut self,
        position: Vec2,
        padding: f32,
        callback: impl FnOnce(&mut Self),
    ) {
        self.layout_with_theme(
            position,
            0.0,
            Some(LayoutTheme {
                background_color: Color::MIDNIGHT_BLUE,
            }),
            callback,
        )
    }

    pub fn panel(&mut self, position: Vec2, callback: impl FnOnce(&mut Self)) {
        self.layout_with_theme(
            position,
            0.0,
            Some(LayoutTheme {
                background_color: Color::MIDNIGHT_BLUE,
            }),
            callback,
        )
    }

    pub fn label(&mut self, text_value: &str) {
        let position = self.get_next_widget_position();
        let text = Text {
            value: text_value.to_owned(),

            ..Default::default()
        };
        self.text(text.clone(), position, None);

        let text_size = self.measure_text(text, None);

        self.get_current_layout_mut().push_widget(text_size);
    }

    pub fn text(&mut self, text: Text, position: Vec2, max_bounds: Option<Vec2>) {
        let font = self.fonts.get(&DEFAULT_FONT_ID).unwrap();

        let rect = Rect {
            min: position,
            max: max_bounds.unwrap_or(self.container_size),
        };
        let text_glyphs = self.font_atlas.queue_text(
            font,
            &text,
            &rect,
            &mut self.texture_atlases,
            &mut self.texture_atlases_images,
            fontdue::layout::CoordinateSystem::PositiveYDown,
        );

        let transform = Transform::from_xyz(rect.min.x, rect.min.y, 0.0);

        for text_glyph in text_glyphs {
            let atlas = self
                .texture_atlases
                .get(&text_glyph.atlas_info.texture_atlas_id)
                .unwrap();

            let current_image_size = atlas.size;
            let scale_factor = 1f32;

            let extracted_transform = transform.compute_matrix()
                * Mat4::from_scale(Vec3::splat(scale_factor.recip()))
                * Mat4::from_translation(text_glyph.position.extend(0.));
            let rect = text_glyph.rect;

            let mut vertices = Vec::new();

            let uvs = [
                Vec2::new(rect.min.x, rect.min.y),
                Vec2::new(rect.max.x, rect.min.y),
                Vec2::new(rect.max.x, rect.max.y),
                Vec2::new(rect.min.x, rect.max.y),
            ]
            .map(|pos| pos / current_image_size);

            let positions = QUAD_VERTEX_POSITIONS.map(|pos| {
                (extracted_transform
                    * ((pos - Vec2::new(-0.5, -0.5)) * rect.size())
                        .extend(0.)
                        .extend(1.))
                .xyx()
                .into()
            });

            for i in 0..QUAD_VERTEX_POSITIONS.len() {
                vertices.push(UiVertex {
                    position: positions[i],
                    tex_coords: uvs[i].into(),
                    color: text.theme.color.as_rgba_f32(),
                });
            }

            let meta = RenderBatchMeta {
                texture_id: text_glyph.atlas_info.texture_atlas_id,
                vertices,
                indices: QUAD_INDICES.to_vec(),
            };

            let layout = self.get_current_layout_mut();

            layout.ui_meta.push(meta);
        }

        // panic!("testing");
    }

    pub fn layout(&mut self, position: Vec2, padding: f32, callback: impl FnOnce(&mut Self)) {
        self.layout_with_theme(position, padding, None, callback)
    }

    pub fn layout_with_theme(
        &mut self,
        position: Vec2,
        padding: f32,
        layout_theme: Option<LayoutTheme>,
        callback: impl FnOnce(&mut Self),
    ) {
        self.begin(position, padding, layout_theme);
        callback(self);
        self.end_layout();
    }

    pub fn begin(&mut self, position: Vec2, padding: f32, theme: Option<LayoutTheme>) {
        let mut layout = Layout::default();
        layout.padding = padding;
        layout.position = position;
        layout.layout_theme = theme;

        self.current_layout.push(layout);
    }

    pub fn begin_with_layout(&mut self, layout: Layout) {
        self.current_layout.push(layout);
    }

    pub fn end_layout(&mut self) {
        let mut layout = self.current_layout.pop().unwrap();

        if let Some(theme) = layout.layout_theme {
            let transform = Transform::from_xyz(layout.position.x, layout.position.y, 1.0);

            let mut vertices = Vec::new();

            let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
                (transform // offset the center point so it renders top left
                    .transform_point(
                        ((quad_pos - Vec2::new(-0.5, -0.5)) * layout.current_size).extend(1.),
                    ))
                .into()
            });

            for i in 0..QUAD_VERTEX_POSITIONS.len() {
                vertices.push(UiVertex {
                    position: positions[i],
                    tex_coords: QUAD_UVS[i].into(),
                    color: theme.background_color.as_rgba_f32(),
                });
            }

            self.queued_layouts.push(RenderBatchMeta {
                texture_id: DEFAULT_TEXTURE_ID,
                vertices,
                indices: QUAD_INDICES.to_vec(),
            });
        }

        self.queued_layouts.append(&mut layout.ui_meta);
    }
}
