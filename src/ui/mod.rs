use bevy_ecs::system::Resource;
use glam::Vec2;
use wgpu::Device;

use crate::{
    color::Color,
    internal_image::Image,
    renderer::{RenderBatchItem, RenderBatchMeta, QUAD_INDICES, QUAD_UVS, QUAD_VERTEX_POSITIONS},
    resources::utils::Assets,
    texture_atlas::TextureAtlas,
    DEFAULT_TEXTURE_ID,
};

use self::button::{Button, ButtonState};

pub mod button;

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

pub trait Widget {
    fn get_size(&self) -> Vec2;

    fn get_render_meta(&self, position_to_render: Vec2) -> RenderBatchMeta<UiVertex>;
}

#[derive(Resource)]
pub struct UiHandler {
    pub queued_layouts: Vec<RenderBatchMeta<UiVertex>>,
    current_layout: Vec<Layout>,
    texture_atlases: Assets<TextureAtlas>,
    images: Assets<Image>,
    default_texture_id: uuid::Uuid,
}

impl UiHandler {
    pub fn new() -> Self {
        // Prepare glyph_brush

        UiHandler {
            queued_layouts: Vec::new(),
            current_layout: Vec::new(),
            images: Assets::new(),
            texture_atlases: Assets::new(),
            default_texture_id: DEFAULT_TEXTURE_ID,
        }
    }

    pub fn push<W: Widget>(&mut self, widget: W) {
        let last_index = self.current_layout.len() - 1;
        let layout = self
            .current_layout
            .get_mut(last_index)
            .expect("Widget needs to be inside layout to render");

        let size = widget.get_size();

        let position_to_render = layout.get_next_position();

        let render_meta = widget.get_render_meta(position_to_render);

        layout.push_widget(size);

        layout.ui_meta.push(render_meta);
    }

    pub fn button(&mut self, button: Button) -> ButtonState {
        let last_index = if self.current_layout.is_empty() {
            0
        } else {
            self.current_layout.len() - 1
        };

        let layout = self
            .current_layout
            .get_mut(last_index)
            .expect("Button needs to be inside layout to render");

        let button_size = Vec2::new(50., 50.);

        let position_to_render = layout.get_next_position();

        layout.push_widget(button_size);

        let mut vertices = Vec::new();

        let positions: [[f32; 3]; 4] =
            QUAD_VERTEX_POSITIONS.map(|quad_pos| (quad_pos + position_to_render).extend(0.).into());

        for i in 0..QUAD_VERTEX_POSITIONS.len() {
            vertices.push(UiVertex {
                position: positions[i],
                tex_coords: QUAD_UVS[i].into(),
                color: Color::WHITE.as_rgba_f32(),
            });
        }

        layout.ui_meta.push(RenderBatchMeta {
            texture_id: self.default_texture_id,
            vertices,
            indices: QUAD_INDICES.to_vec(),
        });

        ButtonState { clicked: false }
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

        // TODO: insert original layout here if it has colour??

        self.queued_layouts.append(&mut layout.ui_meta);
    }
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
    position: Vec2,
    padding: f32,
    pub current_size: Vec2,
    pub(crate) ui_meta: Vec<RenderBatchMeta<UiVertex>>,
}

impl Layout {
    pub fn get_next_position(&self) -> Vec2 {
        match self.layout_type {
            LayoutType::Horizontal => Vec2::new(self.current_size.x, self.position.y),
            LayoutType::Vertical => Vec2::new(self.position.x, self.current_size.y),
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

                Vec2::new(self.current_size.x + size.x, new_y)
            }
            LayoutType::Vertical => {
                let new_x = if size.x > self.current_size.x {
                    size.x
                } else {
                    self.current_size.x
                };

                Vec2::new(new_x, self.current_size.y + size.y)
            }
        }
    }
}
