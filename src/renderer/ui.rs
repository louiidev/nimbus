use std::sync::Arc;

use bevy_ecs::{
    query::Without,
    system::{Query, Res, ResMut},
};
use glam::Vec2;
use wgpu::{
    util::DeviceExt, Extent3d, RenderPass, RenderPipeline, TextureDimension, TextureFormat,
};

use crate::{
    camera::{Camera, CameraUniform},
    components::sprite::Sprite,
    internal_image::{Image, DEFAULT_TEXTURE_FORMAT},
    resources::utils::{Assets, ResourceVec},
    transform::GlobalTransform,
    ui::{UiHandler, UiVertex},
    App, CoreStage, DEFAULT_TEXTURE_ID,
};

use super::{
    plugin_2d::{DefaultImageSampler, SpritePipeline},
    texture::Texture,
    RenderBatchItem, RenderBatchMeta, Renderer, QUAD_UVS, QUAD_VERTEX_POSITIONS,
};

pub fn create_ui_pipeline() {}

pub fn render_ui<'a>(render_pass: &mut RenderPass<'a>, pipeline: RenderPipeline) {}

pub fn prepare_ui_for_batching(
    mut ui_handler: ResMut<UiHandler>,
    renderer: Res<Renderer>,
    sprite_assets: Res<Assets<Texture>>,
    sprite_pipeline: Res<SpritePipeline>,
    default_sampler: Res<DefaultImageSampler>,
    mut layout_batches: ResMut<ResourceVec<RenderBatchItem>>,
    mut camera: Query<(&mut Camera, &mut GlobalTransform)>,
) {
    let mut current_batch_texture_id = uuid::Uuid::new_v4();

    let mut batches: Vec<RenderBatchMeta<UiVertex>> = Vec::new();

    let layout_meta = ui_handler.queued_layouts.drain(..);

    for layout in layout_meta {
        if current_batch_texture_id == layout.texture_id {
            let length = batches.len();

            let current = &mut batches[length - 1];
            let vert_count = current.vertices.len() as u16;
            let indices = layout
                .indices
                .iter()
                .map(|index| index + vert_count)
                .collect();

            current.update(layout.vertices, indices);
        } else {
            current_batch_texture_id = layout.texture_id;
            batches.push(RenderBatchMeta::new(
                layout.texture_id,
                layout.vertices,
                layout.indices,
            ));
        }
    }

    let batches: Vec<RenderBatchItem> = batches
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
            }
        })
        .collect();

    layout_batches.values = batches;
}

impl App {
    pub fn init_ui_renderer(mut self) -> Self {
        self.schedule
            .add_system_to_stage(CoreStage::PrepareRenderer, prepare_ui_for_batching);

        self
    }
}
