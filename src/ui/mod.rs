use bevy_ecs::system::Resource;
use glam::Vec2;
use winit::event::MouseButton;

use crate::{
    components::text::Text,
    font::FontAtlasSet,
    internal_image::Image,
    rect::Rect,
    renderer::{RenderBatchMeta, QUAD_INDICES, QUAD_UVS, QUAD_VERTEX_POSITIONS},
    resources::{inputs::InputController, utils::Assets},
    texture_atlas::TextureAtlas,
    transform::Transform,
    utils::collision,
    DEFAULT_FONT_ID, DEFAULT_TEXTURE_ID,
};

use self::{
    button::Button,
    id::Id,
    layout::Layout,
    widget::{Widget, WidgetResponse},
};

pub mod button;
pub mod id;
pub mod layout;
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
    images: Assets<Image>,
    pub input_controller: InputController,
    pub active_id: Option<Id>,
    pub hover_id: Option<Id>,
    pub font_atlas: FontAtlasSet,
}

impl Default for UiHandler {
    fn default() -> Self {
        UiHandler {
            queued_layouts: Vec::default(),
            current_layout: Vec::default(),
            images: Assets::default(),
            texture_atlases: Assets::new(),
            input_controller: InputController::default(),
            active_id: None,
            hover_id: None,
            font_atlas: FontAtlasSet::default(),
        }
    }
}

impl UiHandler {
    pub fn new() -> Self {
        // Prepare glyph_brush

        UiHandler {
            queued_layouts: Vec::new(),
            current_layout: Vec::new(),
            images: Assets::new(),
            texture_atlases: Assets::new(),
            input_controller: InputController::default(),
            active_id: None,
            hover_id: None,
            font_atlas: FontAtlasSet::default(),
        }
    }

    // pub fn push<W: Widget>(&mut self, widget: W) {
    //     let last_index = self.current_layout.len() - 1;
    //     let layout = self
    //         .current_layout
    //         .get_mut(last_index)
    //         .expect("Widget needs to be inside layout to render");

    //     let size = widget.get_size();

    //     let position_to_render = layout.get_next_position();

    //     let render_meta = widget.get_render_meta(position_to_render);

    //     layout.push_widget(size);

    //     layout.ui_meta.push(render_meta);
    // }

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

    pub fn generate_id(&self) -> Id {
        let last_index = self.current_layout.len() - 1;

        let layout = self.get_current_layout();

        Id::new(format!(
            "Layout_index_{}_ui_count_{}",
            last_index,
            layout.ui_meta.len()
        ))
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
        collision::rect_contains_point(size, position, self.input_controller.mouse_position)
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

    pub fn text(&mut self, text: Text, bounds: (f32, f32), font_size: f32) {
        let font = self.font_atlas.fonts.get(&DEFAULT_FONT_ID).unwrap();

        let text_glyphs = self
            .font_atlas
            .queue_text(&font.font, text, bounds, font_size);

        for text_glyph in text_glyphs {
            let atlas = self
                .font_atlas
                .texture_atlases
                .get(&text_glyph.atlas_info.texture_atlas_id)
                .unwrap();
        }
    }

    pub fn layout<F>(&mut self, position: Vec2, padding: f32, mut callback: F)
    where
        // The closure takes no input and returns nothing.
        F: FnMut(&mut Self),
    {
        self.begin(position, padding);
        callback(self);
        self.end_layout();
    }

    pub fn begin(&mut self, position: Vec2, padding: f32) {
        let mut layout = Layout::default();
        layout.padding = padding;
        layout.position = position;

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
