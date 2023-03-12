use std::sync::Arc;

use bevy_ecs::{
    prelude::Component,
    system::{Query, Res, ResMut},
};
use glam::Vec3;
use hashbrown::HashMap;
use wgpu::{util::DeviceExt, BindGroup, Buffer, RenderPass};

use crate::{
    camera::CameraBindGroupType,
    resources::utils::{Assets, ResourceVec},
    transform::Transform,
    DEFAULT_TEXTURE_ID,
};

use super::{
    mesh_pipeline::MeshPipeline, plugin_2d::DefaultImageSampler, texture::Texture, Renderer,
};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl MeshVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 6]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 8]>()) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Component)]
pub struct Mesh {
    vertices: Vec<MeshVertex>,
    indices: Vec<u16>,
    texture_id: Option<uuid::Uuid>,
}

pub enum MeshPipelineKey {}

pub struct PreparedMeshItem {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    texture_bind_group: BindGroup,
    indices_len: u32,
}

pub fn prepare_mesh_to_render(
    renderer: Res<Renderer>,
    mesh_query: Query<(&Mesh, &Transform)>,
    mesh_pipeline: Res<MeshPipeline>,
    mut prepared_meshes: ResMut<ResourceVec<PreparedMeshItem>>,
    default_sampler: Res<DefaultImageSampler>,
    texture: Res<Assets<Texture>>,
) {
    for (mesh, transform) in mesh_query.iter() {
        let update_vertices = mesh
            .vertices
            .iter()
            .map(|mesh_vertex| {
                let position: [f32; 3] = transform
                    .transform_point(Vec3::from_array(mesh_vertex.position))
                    .into();

                MeshVertex {
                    position,
                    normal: mesh_vertex.normal,
                    uv: mesh_vertex.uv,
                    color: mesh_vertex.color,
                }
            })
            .collect::<Vec<MeshVertex>>();

        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&update_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let texture_id = mesh.texture_id.unwrap_or(DEFAULT_TEXTURE_ID);

        let texture = texture.data.get(&texture_id).unwrap();

        let texture_bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &mesh_pipeline.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&default_sampler.0),
                    },
                ],
                label: Some("diffuse_bind_group"),
            });

        prepared_meshes.values.push(PreparedMeshItem {
            vertex_buffer,
            index_buffer,
            texture_bind_group,
            indices_len: mesh.indices.len() as _,
        });
    }
}

pub fn render_meshes<'a>(
    meshes: &'a Vec<PreparedMeshItem>,
    render_pass: &mut RenderPass<'a>,
    mesh_pipeline: &'a MeshPipeline,
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
        render_pass.set_bind_group(1, &mesh.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..mesh.indices_len, 0, 0..1);
    }
}
