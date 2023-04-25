use std::{collections::HashMap, sync::Arc};

use glam::{Vec2, Vec3};
use wgpu::{util::DeviceExt, BindGroup, Device};

use crate::{
    arena::Arena,
    camera::{self, Camera, CameraUniform},
    components::transform::Transform,
    renderer::{
        pipelines::{Pipeline, PipelineType},
        Renderer,
    },
};

pub fn create_camera_bind_group(
    camera: &Camera,
    device: &Device,
    pipeline: &Pipeline,
) -> BindGroup {
    let projection = camera.projection_matrix();
    let view = camera.transform.compute_matrix();
    let inverse_view = view.inverse();
    let view_projection = projection * inverse_view;
    let camera_uniform = CameraUniform {
        view_proj: view_projection.to_cols_array_2d(),
    };

    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("View Buffer"),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        contents: bytemuck::cast_slice(&[camera_uniform]),
    });

    let camera_2d_bind_group_layout = &pipeline
        .bind_group_layouts
        .get(&crate::renderer::pipelines::BindGroupLayoutType::Camera)
        .unwrap();

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
        label: Some("3d camera bind group"),
        layout: camera_2d_bind_group_layout,
    })
}
