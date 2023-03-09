use std::{cmp::Ordering, sync::Arc};

use bevy_ecs::{
    query::Without,
    system::{Query, Res, ResMut},
};
use glam::Vec2;
use hashbrown::HashMap;
use wgpu::{util::DeviceExt, RenderPass};

use crate::{
    camera::{Camera, CameraUniform, ORTHOGRAPHIC_PROJECTION_BIND_GROUP_ID},
    components::sprite::Sprite,
    resources::utils::{Assets, ResourceVec},
    transform::{GlobalTransform, Transform},
};

use super::{
    plugin_2d::{DefaultImageSampler, SpritePipeline},
    texture::Texture,
    RenderBatchItem, RenderBatchMeta, Renderer, Vertex, QUAD_INDICES, QUAD_UVS,
    QUAD_VERTEX_POSITIONS,
};

pub fn prepare_sprites_for_batching(
    sprite_query: Query<(&Sprite, &GlobalTransform, &Transform), Without<Camera>>,
    renderer: Res<Renderer>,
    sprite_assets: Res<Assets<Texture>>,
    sprite_pipeline: Res<SpritePipeline>,
    default_sampler: Res<DefaultImageSampler>,
    mut sprite_batch: ResMut<ResourceVec<RenderBatchItem>>,
    mut camera: Query<(&mut Camera, &mut GlobalTransform), Without<Sprite>>,
) {
    let (mut camera, global_transform) = camera.get_single_mut().unwrap();

    let projection = camera.projection_matrix();

    let view = global_transform.compute_matrix();
    let inverse_view = view.inverse();
    let view_projection = projection * inverse_view;

    let camera_uniform = CameraUniform {
        view_proj: view_projection.to_cols_array_2d(),
    };

    let camera_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("View Buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&[camera_uniform]),
        });

    let camera_bind_group = renderer
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("sprite camera bind group"),
            layout: &sprite_pipeline.camera_bind_group_layout,
        });

    camera.bind_groups.insert(
        ORTHOGRAPHIC_PROJECTION_BIND_GROUP_ID,
        Arc::new(camera_bind_group),
    );

    let mut current_batch_texture_id = uuid::Uuid::new_v4();

    let mut batches: Vec<RenderBatchMeta<Vertex>> = Vec::new();

    let mut query = sprite_query
        .iter()
        .collect::<Vec<(&Sprite, &GlobalTransform, &Transform)>>();

    query.sort_unstable_by(
        |a, b| match a.2.translation.z.partial_cmp(&b.2.translation.z) {
            Some(Ordering::Equal) | None => a.0.texture_id.cmp(&b.0.texture_id),
            Some(other) => other,
        },
    );

    for (sprite, transform, _) in query {
        let mut uvs = QUAD_UVS;

        let mut vertices = Vec::new();

        let texture = sprite_assets.data.get(&sprite.texture_id).unwrap();
        let current_image_size =
            Vec2::new(texture.dimensions.0 as f32, texture.dimensions.1 as f32);

        // By default, the size of the quad is the size of the texture
        let mut quad_size = current_image_size;

        // If a rect is specified, adjust UVs and the size of the quad
        if let Some(rect) = sprite.texture_rect {
            let rect_size = rect.size();
            for uv in &mut uvs {
                *uv = (rect.min + *uv * rect_size) / current_image_size;
            }
            quad_size = rect_size;
        }

        // Override the size if a custom one is specified
        if let Some(custom_size) = sprite.custom_size {
            quad_size = custom_size;
        }

        let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
            transform
                .transform_point(((quad_pos - sprite.anchor.as_vec()) * quad_size).extend(0.))
                .into()
        });

        for i in 0..QUAD_VERTEX_POSITIONS.len() {
            vertices.push(Vertex {
                position: positions[i],
                uv: uvs[i].into(),
                color: sprite.color.into(),
            });
        }

        if current_batch_texture_id == sprite.texture_id {
            let length = batches.len();

            let current = &mut batches[length - 1];
            let vert_count = current.vertices.len() as u16;
            let indices = QUAD_INDICES.map(|index| index + vert_count);

            current.update(vertices, indices.to_vec());
        } else {
            current_batch_texture_id = sprite.texture_id;
            batches.push(RenderBatchMeta::new(
                sprite.texture_id,
                vertices,
                QUAD_INDICES.to_vec(),
            ));
        }
    }

    let mut sprite_batches: Vec<RenderBatchItem> = batches
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

            let texture = sprite_assets.data.get(&batch.texture_id).unwrap();

            let texture_bind_group =
                renderer
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &sprite_pipeline.texture_bind_group_layout,
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

            RenderBatchItem {
                vertex_buffer,
                index_buffer,
                texture_bind_group,
                indices_len: batch.indices.len() as _,
                camera_bind_group_id: ORTHOGRAPHIC_PROJECTION_BIND_GROUP_ID,
            }
        })
        .collect();

    sprite_batch.values.append(&mut sprite_batches);
}

pub fn render_sprite_batches<'a>(
    sprite_batch: &'a Vec<RenderBatchItem>,
    render_pass: &mut RenderPass<'a>,
    sprite_pipeline: &'a SpritePipeline,
    camera_bind_group: &'a HashMap<u8, Arc<wgpu::BindGroup>>,
) {
    render_pass.set_pipeline(&sprite_pipeline.render_pipeline);

    for batch in sprite_batch {
        render_pass.set_bind_group(
            0,
            camera_bind_group.get(&batch.camera_bind_group_id).unwrap(),
            &[],
        );
        render_pass.set_bind_group(1, &batch.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
        render_pass.set_index_buffer(batch.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..batch.indices_len, 0, 0..1);
    }
}
