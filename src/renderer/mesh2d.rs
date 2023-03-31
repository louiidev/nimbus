use std::collections::HashMap;

use glam::Vec2;
use wgpu::{
    include_wgsl, BindGroup, BlendState, Buffer, FragmentState, FrontFace, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor, VertexState,
};

use crate::{arena::ArenaId, camera::CameraBindGroupType, components::color::Color};

use super::{
    pipelines::{BindGroupLayoutType, Pipeline},
    Renderer,
};

pub const QUAD_INDICES: [u16; 6] = [0, 2, 3, 0, 1, 2];

pub const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
    Vec2::new(-0.5, -0.5),
    Vec2::new(0.5, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

pub const QUAD_UVS: [Vec2; 4] = [
    Vec2::new(0., 1.),
    Vec2::new(1., 1.),
    Vec2::new(1., 0.),
    Vec2::new(0., 0.),
];

#[derive(Debug)]
pub struct Mesh2d {
    pub(crate) texture_id: ArenaId,
    pub(crate) vertices: Vec<Vertex2D>,
    pub(crate) indices: Vec<u16>,
}

impl Mesh2d {
    pub fn new(texture_id: ArenaId, vertices: Vec<Vertex2D>, indices: Vec<u16>) -> Self {
        Self {
            texture_id,
            vertices,
            indices,
        }
    }

    pub fn update(&mut self, mut vertices: Vec<Vertex2D>, mut indices: Vec<u16>) {
        self.vertices.append(&mut vertices);
        self.indices.append(&mut indices);
    }

    pub fn rect(position: Vec2, size: Vec2, color: Color) -> Self {
        let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
            (position + ((quad_pos - Vec2::new(-0.5, -0.5)) * size))
                .extend(0.)
                .into()
        });

        let vertices: Vec<Vertex2D> = positions
            .iter()
            .enumerate()
            .map(|(index, vertex_position)| Vertex2D {
                position: *vertex_position,
                uv: QUAD_UVS[index].into(),
                color: color.into(),
            })
            .collect();

        Self {
            texture_id: ArenaId::first(),
            vertices,
            indices: QUAD_INDICES.to_vec(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex2D {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex2D>() as wgpu::BufferAddress,
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
                    offset: (std::mem::size_of::<[f32; 5]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct PreparedRenderItem {
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) texture_bind_group: BindGroup,
    pub(crate) indices_len: u32,
    pub(crate) camera_bind_group_id: CameraBindGroupType,
}

pub fn setup_mesh2d_pipeline(renderer: &Renderer) -> Pipeline {
    let shader = renderer
        .device
        .create_shader_module(include_wgsl!("../assets/shaders/2d.wgsl"));

    let texture_bind_group_layout =
        renderer
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

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
                bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

    let binding = [Some(wgpu::ColorTargetState {
        format: renderer.surface_config.format,
        blend: Some(BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
    })];
    let descriptor = RenderPipelineDescriptor {
        vertex: VertexState {
            entry_point: "vertex",
            buffers: &[Vertex2D::desc()],
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
            topology: PrimitiveTopology::TriangleList,
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

    let bind_group_layouts = HashMap::from([
        (BindGroupLayoutType::Camera, camera_bind_group_layout),
        (BindGroupLayoutType::Texture, texture_bind_group_layout),
    ]);

    Pipeline {
        render_pipeline: renderer.device.create_render_pipeline(&descriptor),
        bind_group_layouts,
    }
}
