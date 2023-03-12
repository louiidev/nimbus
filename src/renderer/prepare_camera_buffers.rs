use std::sync::Arc;

use bevy_ecs::{
    query::Without,
    system::{Query, Res},
};
use wgpu::util::DeviceExt;

use crate::{
    camera::{Camera, CameraBindGroupType, CameraUniform},
    components::sprite::Sprite,
    transform::GlobalTransform,
};

use super::{mesh_pipeline::MeshPipeline, Renderer};

pub fn prepare_camera_3d_bindgroup(
    renderer: Res<Renderer>,
    mesh_pipeline: Res<MeshPipeline>,
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
            label: Some("3d camera bind group"),
            layout: &mesh_pipeline.camera_bind_group_layout,
        });

    camera.bind_groups.insert(
        CameraBindGroupType::Perspective,
        Arc::new(camera_bind_group),
    );
}
