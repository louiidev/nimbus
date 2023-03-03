use std::sync::Arc;

use bevy_ecs::{
    query::Without,
    system::{Query, Res, ResMut},
};
use glam::Vec2;
use wgpu::util::DeviceExt;

use crate::{
    camera::{Camera, CameraUniform, ORTHOGRAPHIC_PROJECTION_BIND_GROUP_ID},
    components::text::Text,
    font::FontData,
    font_atlas::FontAtlasSet,
    internal_image::Image,
    rect::Rect,
    resources::utils::{Assets, ResourceVec},
    texture_atlas::TextureAtlas,
    transform::GlobalTransform,
};

use super::{
    plugin_2d::{DefaultImageSampler, SpritePipeline},
    texture::Texture,
    RenderBatchItem, RenderBatchMeta, Renderer, Vertex, QUAD_INDICES, QUAD_UVS,
    QUAD_VERTEX_POSITIONS,
};

pub fn prepare_text_for_batching(
    text_query: Query<(&Text, &mut GlobalTransform), Without<Camera>>,
    renderer: Res<Renderer>,
    mut font_atlas_set: ResMut<FontAtlasSet>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    textures: ResMut<Assets<Texture>>,
    mut images: ResMut<Assets<Image>>,
    fonts: Res<Assets<FontData>>,
    sprite_pipeline: Res<SpritePipeline>,
    default_sampler: Res<DefaultImageSampler>,
    mut sprite_batch: ResMut<ResourceVec<RenderBatchItem>>,
    mut camera: Query<(&mut Camera, &mut GlobalTransform), Without<Text>>,
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

    for (text, transform) in text_query.iter() {
        let font = fonts.get(&text.font_id).unwrap();
        let text_glyphs = font_atlas_set.queue_text(
            &font,
            text,
            Rect::default(),
            &mut texture_atlases,
            &mut images,
            fontdue::layout::CoordinateSystem::PositiveYDown,
        );

        for text_glyph in text_glyphs {
            let mut vertices = Vec::new();
            let atlas = texture_atlases
                .get(&text_glyph.atlas_info.texture_atlas_id)
                .unwrap();

            let current_image_size = atlas.size;

            let uvs = QUAD_UVS.map(|pos| pos / current_image_size);

            let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
                transform
                    .transform_point(
                        ((quad_pos - Vec2::new(-0.5, -0.5)) * text_glyph.rect.size()).extend(0.),
                    )
                    .into()
            });

            for i in 0..QUAD_VERTEX_POSITIONS.len() {
                vertices.push(Vertex {
                    position: positions[i],
                    uv: uvs[i].into(),
                    color: text.theme.color.as_rgba_f32(),
                });
            }

            if current_batch_texture_id == text_glyph.atlas_info.texture_atlas_id {
                let length = batches.len();

                let current = &mut batches[length - 1];
                let vert_count = current.vertices.len() as u16;
                let indices = QUAD_INDICES.map(|index| index + vert_count);

                current.update(vertices, indices.to_vec());
            } else {
                current_batch_texture_id = text_glyph.atlas_info.texture_atlas_id;
                batches.push(RenderBatchMeta::new(
                    text_glyph.atlas_info.texture_atlas_id,
                    vertices,
                    QUAD_INDICES.to_vec(),
                ));
            }
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

            let texture = textures.data.get(&batch.texture_id).unwrap();

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
