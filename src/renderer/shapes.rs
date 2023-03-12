use bevy_ecs::{
    query::Without,
    system::{Query, Res, ResMut},
};
use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::{
    camera::{Camera, CameraBindGroupType},
    components::shapes::Shape,
    resources::utils::{Assets, ResourceVec},
    transform::{GlobalTransform, Transform},
    DEFAULT_TEXTURE_ID,
};

use super::{
    plugin_2d::{DefaultImageSampler, SpritePipeline},
    texture::Texture,
    PreparedRenderItem, RenderBatchMeta, Renderer, Vertex, QUAD_UVS,
};

pub fn prepare_shapes_for_batching(
    shape_query: Query<(&Shape, &GlobalTransform, &Transform), Without<Camera>>,
    renderer: Res<Renderer>,
    sprite_assets: Res<Assets<Texture>>,
    sprite_pipeline: Res<SpritePipeline>,
    default_sampler: Res<DefaultImageSampler>,
    mut sprite_batch: ResMut<ResourceVec<PreparedRenderItem>>,
) {
    let mut render_batch_meta = RenderBatchMeta::new(DEFAULT_TEXTURE_ID, Vec::new(), Vec::new());

    for (shape, global_transform, transform) in shape_query.iter() {
        let mut vertices = Vec::new();
        let uvs = QUAD_UVS;

        let positions: Vec<Vec3> = shape
            .vertices
            .iter()
            .map(|quad_pos| {
                global_transform.transform_point(
                    (*quad_pos - shape.anchor.as_vec().extend(0.)) * transform.scale,
                )
            })
            .collect();

        for i in 0..shape.vertices.len() {
            vertices.push(Vertex {
                position: positions[i].to_array(),
                uv: shape.tex_coords[i].into(),
                color: shape.color.into(),
            });
        }

        let vert_count = render_batch_meta.vertices.len() as u16;
        let indices = shape
            .indicies
            .iter()
            .map(|index| index + vert_count)
            .collect::<Vec<u16>>();

        render_batch_meta.update(vertices, indices);
    }

    let vertex_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&render_batch_meta.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let index_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&render_batch_meta.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

    let texture = sprite_assets
        .data
        .get(&render_batch_meta.texture_id)
        .unwrap();

    let texture_bind_group = renderer
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

    let batch_item = PreparedRenderItem {
        vertex_buffer,
        index_buffer,
        texture_bind_group,
        indices_len: render_batch_meta.indices.len() as _,
        camera_bind_group_id: CameraBindGroupType::Orthographic,
    };

    sprite_batch.values.push(batch_item);
}
