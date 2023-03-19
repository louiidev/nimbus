use std::cmp::Ordering;

use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::{
    areana::ArenaId,
    camera::CameraBindGroupType,
    components::{sprite::Sprite, transform::Transform},
    renderer::{
        mesh2d::{
            Mesh2d, PreparedRenderItem, SpriteVertex, QUAD_INDICES, QUAD_UVS, QUAD_VERTEX_POSITIONS,
        },
        pipelines::PipelineType,
        Renderer,
    },
};

pub fn prepare_mesh2d_for_batching(renderer: &mut Renderer) -> Vec<PreparedRenderItem> {
    let mut meshes = renderer
        .render_batch_2d
        .drain(0..)
        .collect::<Vec<(Mesh2d<SpriteVertex>, Vec3)>>();
    meshes.sort_unstable_by(|a, b| match a.1.z.partial_cmp(&b.1.z) {
        Some(Ordering::Equal) | None => a.0.texture_id.cmp(&b.0.texture_id),
        Some(other) => other,
    });

    let mut current_batch_texture_id = ArenaId::default();
    let mut batches: Vec<Mesh2d<SpriteVertex>> = Vec::new();

    for (sprite, transform) in meshes {
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

    let mut sprite_batches: Vec<PreparedRenderItem> = batches
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
                        layout: &pipeline.texture_bind_group_layout,
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
                camera_bind_group_id: CameraBindGroupType::Orthographic,
            }
        })
        .collect();

    sprite_batches
}
