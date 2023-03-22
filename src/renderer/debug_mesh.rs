use std::collections::HashMap;

use glam::{Vec2, Vec3};
use wgpu::{
    include_wgsl, Buffer, FragmentState, FrontFace, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipelineDescriptor, VertexState,
};

use crate::components::{color::Color, rect::Rect};

use super::{
    mesh2d::QUAD_INDICES,
    pipelines::{BindGroupLayoutType, Pipeline},
    Renderer,
};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct DebugMeshVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl DebugMeshVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<DebugMeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>()) as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct DebugMesh {
    pub(crate) vertices: Vec<DebugMeshVertex>,
    pub(crate) indices: Vec<u16>,
}

impl DebugMesh {
    pub fn rect(rect: &Rect, color: Color) -> Self {
        let color = color.as_rgba_f32();

        let vertices = vec![
            DebugMeshVertex {
                position: rect.min.extend(1.).into(),
                color,
            },
            DebugMeshVertex {
                position: Vec3::new(rect.max.x, rect.min.y, 0.).into(),
                color,
            },
            DebugMeshVertex {
                position: rect.max.extend(1.).into(),
                color,
            },
            DebugMeshVertex {
                position: Vec3::new(rect.min.x, rect.max.y, 0.).into(),
                color,
            },
            DebugMeshVertex {
                position: rect.min.extend(1.).into(),
                color,
            },
        ];

        DebugMesh {
            vertices,
            indices: QUAD_INDICES.to_vec(),
        }
    }

    pub fn line(p1: Vec2, p2: Vec2, color: Color) -> Self {
        let color = color.as_rgba_f32();
        let vertices = vec![
            DebugMeshVertex {
                position: p1.extend(1.).into(),
                color,
            },
            DebugMeshVertex {
                position: p2.extend(1.).into(),
                color,
            },
        ];

        DebugMesh {
            vertices,
            indices: QUAD_INDICES.to_vec(),
        }
    }
}

pub struct PreparedDebugMeshItem {
    pub(crate) vertex_buffer: Buffer,
    pub(crate) vertices_len: u32,
}

pub fn setup_debug_mesh_pipeline(renderer: &Renderer) -> Pipeline {
    let shader = renderer
        .device
        .create_shader_module(include_wgsl!("../assets/shaders/debug_mesh.wgsl"));

    let camera_bind_group_layout =
        renderer
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

    let render_pipeline_layout =
        renderer
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("2D Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

    let binding = [Some(wgpu::ColorTargetState {
        format: renderer.surface_config.format,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    })];

    let descriptor = RenderPipelineDescriptor {
        vertex: VertexState {
            entry_point: "vertex",
            buffers: &[DebugMeshVertex::desc()],
            module: &shader,
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fragment",
            targets: &binding,
        }),
        layout: Some(&render_pipeline_layout),
        primitive: PrimitiveState {
            front_face: FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
            topology: PrimitiveTopology::LineStrip,
            strip_index_format: None,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,                         // 2.
            mask: !0,                         // 3.
            alpha_to_coverage_enabled: false, // 4.
        },
        label: Some("sprite_pipeline"),
        multiview: None,
    };

    let bind_group_layouts =
        HashMap::from([(BindGroupLayoutType::Camera, camera_bind_group_layout)]);

    Pipeline {
        render_pipeline: renderer.device.create_render_pipeline(&descriptor),
        bind_group_layouts,
    }
}
