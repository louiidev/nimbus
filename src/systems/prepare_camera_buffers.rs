use std::sync::Arc;

use glam::{Vec2, Vec3};
use wgpu::util::DeviceExt;

use crate::{
    camera::{Camera, CameraBindGroupType, CameraUniform},
    components::transform::Transform,
    renderer::{pipelines::PipelineType, Renderer},
};

pub fn prepare_camera_buffers(renderer: &Renderer, camera: &mut Camera) {
    let projection = camera.projection_matrix();

    let view = camera.transform.compute_matrix();
    let inverse_view = view.inverse();
    let view_projection = projection * inverse_view;

    let camera_uniform = CameraUniform {
        view_proj: view_projection.to_cols_array_2d(),
    };

    // 2D
    let pipeline = renderer
        .render_pipelines
        .get(&PipelineType::Mesh2d)
        .unwrap();
    let camera_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("View Buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&[camera_uniform]),
        });

    let camera_2d_bind_group_layout = &pipeline
        .bind_group_layouts
        .get(&crate::renderer::pipelines::BindGroupLayoutType::Camera)
        .unwrap();

    let camera_bind_group = renderer
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("3d camera bind group"),
            layout: camera_2d_bind_group_layout,
        });

    camera.bind_groups.insert(
        CameraBindGroupType::Orthographic,
        Arc::new(camera_bind_group),
    );

    // 2D UI
    let projection = camera.projection_matrix_ui(renderer.get_viewport());
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
            label: Some("3d camera bind group"),
            layout: camera_2d_bind_group_layout,
        });

    camera.bind_groups.insert(
        CameraBindGroupType::OrthographicUI,
        Arc::new(camera_bind_group),
    );

    //3D Debug
    let pipeline = renderer
        .render_pipelines
        .get(&PipelineType::DebugMesh)
        .unwrap();

    let projection = camera.projection_matrix();

    let view = camera.transform.compute_matrix();
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
            layout: pipeline
                .bind_group_layouts
                .get(&crate::renderer::pipelines::BindGroupLayoutType::Camera)
                .unwrap(),
        });

    camera.bind_groups.insert(
        CameraBindGroupType::Perspective,
        Arc::new(camera_bind_group),
    );

    //3D Debug Mesh
}
