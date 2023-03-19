use std::sync::Arc;

use bevy_ecs::{
    prelude::Component,
    system::{Res, ResMut, Resource},
};
use hashbrown::HashMap;
use wgpu::{
    include_wgsl, util::DeviceExt, BindGroupLayout, Buffer, FragmentState, FrontFace, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    VertexState,
};

use crate::{camera::CameraBindGroupType, resources::utils::ResourceVec};

use super::Renderer;

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

#[derive(Component)]
pub struct DebugMesh {
    pub(crate) vertices: Vec<DebugMeshVertex>,
    pub(crate) indices: Vec<u16>,
}

pub struct PreparedDebugMeshItem {
    vertex_buffer: Buffer,
    vertices_len: u32,
}

pub fn prepare_debug_mesh_to_render(
    renderer: Res<Renderer>,
    mut debug_meshes: ResMut<ResourceVec<DebugMesh>>,
    mut prepared_meshes: ResMut<ResourceVec<PreparedDebugMeshItem>>,
) {
    for mesh in &debug_meshes.values {
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // let index_buffer = renderer
        //     .device
        //     .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: Some("Index Buffer"),
        //         contents: bytemuck::cast_slice(&mesh.indices),
        //         usage: wgpu::BufferUsages::INDEX,
        //     });

        prepared_meshes.values.push(PreparedDebugMeshItem {
            vertex_buffer,
            vertices_len: mesh.vertices.len() as _,
        });
    }

    debug_meshes.values.clear();
}

pub fn render_debug_meshes<'a>(
    meshes: &'a Vec<PreparedDebugMeshItem>,
    render_pass: &mut RenderPass<'a>,
    mesh_pipeline: &'a DebugMeshPipeline,
    camera_bind_group: &'a HashMap<CameraBindGroupType, Arc<wgpu::BindGroup>>,
) {
    render_pass.set_pipeline(&mesh_pipeline.render_pipeline);

    render_pass.set_bind_group(
        0,
        camera_bind_group
            .get(&CameraBindGroupType::Perspective)
            .unwrap(),
        &[],
    );

    for mesh in meshes {
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        // render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw(0..mesh.vertices_len, 0..1)
        // render_pass.draw_indexed(0..mesh.indices_len, 0, 0..1);
    }
}

#[derive(Resource)]
pub struct DebugMeshPipeline {
    pub render_pipeline: RenderPipeline,
    pub camera_bind_group_layout: BindGroupLayout,
}

impl DebugMeshPipeline {
    pub fn new(renderer: &Renderer) -> Self {
        let shader = renderer
            .device
            .create_shader_module(include_wgsl!("../shaders/debug_mesh.wgsl"));

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
                    label: Some("Mesh Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let binding = [Some(wgpu::ColorTargetState {
            format: renderer.config.format,
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
            label: Some("debug_pipeline"),
            multiview: None,
        };

        Self {
            render_pipeline: renderer.device.create_render_pipeline(&descriptor),
            camera_bind_group_layout,
        }
    }
}
