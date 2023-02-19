use bevy_ecs::system::Resource;
use glam::Vec2;
use winit::event::MouseButton;

use crate::{
    color::Color,
    components::text::Text,
    font::{Font, FontAtlasSet},
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
    pub texture_atlases_images: Assets<Image>,
    pub input_controller: InputController,
    pub active_id: Option<Id>,
    pub hover_id: Option<Id>,
    pub font_atlas: FontAtlasSet,
    pub(crate) fonts: Assets<Font>,
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
        }
    }
}

impl UiHandler {
    pub fn new() -> Self {
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

    pub fn generate_id(&self) -> Id {
        let last_index = self.current_layout.len() - 1;

        let layout = self.get_current_layout();

        Id::new(format!(
            "Layout_index_{}_ui_count_{}",
            last_index,
            layout.ui_meta.len()
        ))
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
        let response = button.ui(self);

        response
    }

    pub fn image(&mut self) {
        for (id, image) in self.texture_atlases_images.data.iter() {
            let position = Vec2::new(200., 200.);

            let image_size = Vec2::new(
                image.texture_descriptor.size.width as f32,
                image.texture_descriptor.size.height as f32,
            );

            let transform = Transform::from_xyz(position.x, position.y, 1.0);

            let mut vertices = Vec::new();

            let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
                (transform // offset the center point so it renders top left
                        .transform_point(
                            ((quad_pos - Vec2::new(-0.5, -0.5)) * image_size).extend(1.),
                        ))
                    .into()
            });

            for i in 0..QUAD_VERTEX_POSITIONS.len() {
                vertices.push(UiVertex {
                    position: positions[i],
                    tex_coords: QUAD_UVS[i].into(),
                    color: Color::WHITE.as_rgba_f32(),
                });
            }
            let last_index = self.current_layout.len() - 1;
            let layout = self
                .current_layout
                .get_mut(last_index)
                .expect("Button needs to be inside layout to render");

            let meta = RenderBatchMeta {
                texture_id: *id,
                vertices,
                indices: QUAD_INDICES.to_vec(),
            };

            layout.ui_meta.push(meta);
        }
    }

    pub fn text(&mut self, text: Text, rect: Rect) {
        let font = self.fonts.get(&DEFAULT_FONT_ID).unwrap();
        let mut uvs = QUAD_UVS;
        let text_glyphs = self.font_atlas.queue_text(
            &font.font,
            &text,
            rect,
            text.theme.font_size,
            &mut self.texture_atlases,
            &mut self.texture_atlases_images,
        );

        for text_glyph in text_glyphs {
            let atlas = self
                .texture_atlases
                .get(&text_glyph.atlas_info.texture_atlas_id)
                .unwrap();

            let current_image_size = atlas.size;

            let index = text_glyph.atlas_info.glyph_index;
            let rect = atlas.textures[index];
            let transform = Transform::from_xyz(text_glyph.position.x, text_glyph.position.y, 1.0);

            let mut vertices = Vec::new();

            let (uvs, quad_size) = {
                let rect_size = rect.size();
                for uv in &mut uvs {
                    *uv = (rect.min + *uv * rect_size) / current_image_size;
                }

                (uvs, rect_size)
            };

            let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
                return transform
                    .transform_point(((quad_pos - Vec2::new(-0.5, -0.5)) * quad_size).extend(0.))
                    .into();
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
