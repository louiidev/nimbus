use std::sync::Arc;

use bevy_ecs::{
    query::Without,
    system::{Query, Res, ResMut, Resource},
};
use glam::Vec2;
use wgpu::{
    include_wgsl, util::DeviceExt, BindGroup, BindGroupLayout, Buffer, FragmentState, FrontFace,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPass, RenderPipeline,
};

use crate::{
    camera::Camera,
    components::sprite::Sprite,
    resources::utils::{Asset, ResourceVec},
    transform::GlobalTransform,
};

use super::{
    plugin_2d::{DefaultImageSampler, SpritePipeline},
    texture::Texture,
    Renderer, Vertex,
};

const QUAD_INDICES: [u16; 6] = [0, 2, 3, 0, 1, 2];

const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
    Vec2::new(-0.5, -0.5),
    Vec2::new(0.5, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

const QUAD_UVS: [Vec2; 4] = [
    Vec2::new(0., 1.),
    Vec2::new(1., 1.),
    Vec2::new(1., 0.),
    Vec2::new(0., 0.),
];

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

#[derive(Resource)]
pub struct SpriteBatch {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    texture_bind_group: BindGroup,
    indices_len: u32,
}

pub struct TempSpriteBatch {
    texture_id: uuid::Uuid,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl TempSpriteBatch {
    pub fn new(texture_id: uuid::Uuid, vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self {
            texture_id,
            vertices,
            indices,
        }
    }

    pub fn update(&mut self, mut vertices: Vec<Vertex>, mut indices: Vec<u16>) {
        self.vertices.append(&mut vertices);
        self.indices.append(&mut indices);

        dbg!(&self.indices);
    }
}

pub fn prepare_sprites_for_batching(
    sprite_query: Query<(&Sprite, &mut GlobalTransform), Without<Camera>>,
    renderer: Res<Renderer>,
    sprite_assets: Res<Asset<Texture>>,
    sprite_pipeline: Res<SpritePipeline>,
    default_sampler: Res<DefaultImageSampler>,
    mut sprite_batch: ResMut<ResourceVec<SpriteBatch>>,
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

    camera.bind_group = Some(Arc::new(camera_bind_group));

    let mut current_batch_texture_id = uuid::Uuid::new_v4();

    let mut batches: Vec<TempSpriteBatch> = Vec::new();

    for (sprite, transform) in sprite_query.iter() {
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
            batches.push(TempSpriteBatch::new(
                sprite.texture_id,
                vertices,
                QUAD_INDICES.to_vec(),
            ));
        }
    }

    let sprite_batches: Vec<SpriteBatch> = batches
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

            SpriteBatch {
                vertex_buffer,
                index_buffer,
                texture_bind_group,
                indices_len: batch.indices.len() as _,
            }
        })
        .collect();

    sprite_batch.value = sprite_batches;
}

pub fn render_sprite_batches<'a>(
    sprite_batch: &'a Vec<SpriteBatch>,
    render_pass: &mut RenderPass<'a>,
    sprite_pipeline: &'a SpritePipeline,
    camera_bind_group: &'a BindGroup,
) {
    for batch in sprite_batch {
        render_pass.set_pipeline(&sprite_pipeline.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, &batch.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
        render_pass.set_index_buffer(batch.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..batch.indices_len, 0, 0..1);
    }
}
