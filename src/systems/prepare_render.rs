use std::cmp::Ordering;

use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::{
    arena::ArenaId,
    renderer::{
        debug_mesh::PreparedDebugMeshItem,
        mesh2d::{Mesh2d, PreparedRenderItem},
        pipelines::PipelineType,
        ui::Ui,
        Renderer,
    },
};

pub fn prepare_mesh2d_for_batching(renderer: &mut Renderer) -> Vec<PreparedRenderItem> {
    let mut meshes = renderer
        .meshes2d
        .drain(0..)
        .collect::<Vec<(Mesh2d, Vec3)>>();
    meshes.sort_unstable_by(|a, b| match a.1.z.partial_cmp(&b.1.z) {
        Some(Ordering::Equal) | None => a.0.texture_id.cmp(&b.0.texture_id),
        Some(other) => other,
    });

    let mut current_batch_texture_id = ArenaId::default();
    let mut batches: Vec<Mesh2d> = Vec::new();

    for (sprite, _) in meshes {
        if current_batch_texture_id == sprite.texture_id {
            let length = batches.len();

            let current = &mut batches[length - 1];
            let vert_count = current.vertices.len() as u16;
            let indices = sprite
                .indices
                .iter()
                .map(|index| index + vert_count)
                .collect::<Vec<u16>>();

            current.update(sprite.vertices, indices);
        } else {
            current_batch_texture_id = sprite.texture_id;
            batches.push(sprite);
        }
    }

    batches
        .iter()
        .map(|batch| {
            let vertex_buffer =
                renderer
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

            let index_buffer =
                renderer
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&batch.indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });

            let texture = renderer.textures.get(batch.texture_id).unwrap();

            let sampler = &renderer.get_texture_sampler(texture.sampler);

            let pipeline = renderer
                .render_pipelines
                .get(&PipelineType::Mesh2d)
                .unwrap();

            let texture_bind_group =
                renderer
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: pipeline
                            .bind_group_layouts
                            .get(&crate::renderer::pipelines::BindGroupLayoutType::Texture)
                            .unwrap(),
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&texture.view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(sampler),
                            },
                        ],
                        label: Some("diffuse_bind_group"),
                    });

            PreparedRenderItem {
                vertex_buffer,
                index_buffer,
                texture_bind_group,
                indices_len: batch.indices.len() as _,
            }
        })
        .collect()
}

pub fn prepare_ui_for_batching(ui: &mut Ui, renderer: &mut Renderer) -> Vec<PreparedRenderItem> {
    let mut current_batch_texture_id = ArenaId::default();
    let mut batches: Vec<Mesh2d> = Vec::new();

    let meshes = ui.render_meta.drain(0..).collect::<Vec<Mesh2d>>();
    ui.reset();

    for mesh in meshes {
        if current_batch_texture_id == mesh.texture_id {
            let length = batches.len();

            let current = &mut batches[length - 1];
            let vert_count = current.vertices.len() as u16;
            let indices = mesh
                .indices
                .iter()
                .map(|index| index + vert_count)
                .collect::<Vec<u16>>();

            current.update(mesh.vertices, indices);
        } else {
            current_batch_texture_id = mesh.texture_id;
            batches.push(mesh);
        }
    }

    batches
        .iter()
        .map(|batch| {
            let vertex_buffer =
                renderer
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

            let index_buffer =
                renderer
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&batch.indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });

            let texture = renderer.textures.get(batch.texture_id).unwrap();

            let sampler = &renderer.get_texture_sampler(texture.sampler);

            let pipeline = renderer
                .render_pipelines
                .get(&PipelineType::Mesh2d)
                .unwrap();

            let texture_bind_group =
                renderer
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: pipeline
                            .bind_group_layouts
                            .get(&crate::renderer::pipelines::BindGroupLayoutType::Texture)
                            .unwrap(),
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&texture.view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(sampler),
                            },
                        ],
                        label: Some("diffuse_bind_group"),
                    });

            PreparedRenderItem {
                vertex_buffer,
                index_buffer,
                texture_bind_group,
                indices_len: batch.indices.len() as _,
            }
        })
        .collect()
}

pub fn prepare_debug_mesh_for_batching(renderer: &mut Renderer) -> Vec<PreparedDebugMeshItem> {
    renderer
        .debug_meshes
        .drain(0..)
        .map(|debug_mesh| {
            let vertex_buffer =
                renderer
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&debug_mesh.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

            PreparedDebugMeshItem {
                vertex_buffer,
                vertices_len: debug_mesh.vertices.len() as _,
            }
        })
        .collect()
}
