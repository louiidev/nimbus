use std::sync::Arc;

use bevy_ecs::{
    schedule::IntoSystemConfig,
    system::{Query, Res, ResMut},
};
use glam::{Vec2, Vec3};
use wgpu::util::DeviceExt;

use crate::{
    camera::{Camera, CameraBindGroupType, CameraUniform},
    resources::utils::{Assets, ResourceVec},
    transform::Transform,
    ui::{UiHandler, UiVertex},
    App, CoreSet,
};

use super::{
    plugin_2d::{DefaultImageSampler, SpritePipeline},
    texture::Texture,
    PreparedRenderItem, RenderBatchMeta, Renderer,
};

pub fn prepare_ui_for_batching(
    mut ui_handler: ResMut<UiHandler>,
    renderer: Res<Renderer>,
    sprite_assets: Res<Assets<Texture>>,
    sprite_pipeline: Res<SpritePipeline>,
    default_sampler: Res<DefaultImageSampler>,
    mut layout_batches: ResMut<ResourceVec<PreparedRenderItem>>,
    mut camera: Query<(&mut Camera)>,
) {
    let mut camera = camera.get_single_mut().unwrap();

    let projection = camera.projection_matrix_ui(Vec2::new(
        renderer.size.width as f32,
        renderer.size.height as f32,
    ));
    let transform = Transform {
        translation: Vec3::new(0., 0., 999.),
        ..Default::default()
    };
    let inverse_view = transform.compute_matrix().inverse();
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
        CameraBindGroupType::OrthographicUI,
        Arc::new(camera_bind_group),
    );

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

    let mut batches: Vec<PreparedRenderItem> = batches
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

            let texture = sprite_assets
                .data
                .get(&batch.texture_id)
                .unwrap_or_else(|| panic!("Missing texture id = {}", &batch.texture_id));

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

            PreparedRenderItem {
                vertex_buffer,
                index_buffer,
                texture_bind_group,
                indices_len: batch.indices.len() as _,
                camera_bind_group_id: CameraBindGroupType::OrthographicUI,
            }
        })
        .collect();

    layout_batches.values.append(&mut batches);
}

impl App {
    pub fn init_ui_renderer(mut self) -> Self {
        self.schedule
            .add_system(prepare_ui_for_batching.in_base_set(CoreSet::PrepareRenderer));

        self
    }
}
