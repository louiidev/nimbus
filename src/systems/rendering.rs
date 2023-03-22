use std::{collections::HashMap, sync::Arc};

use wgpu::RenderPass;

use crate::{
    camera::CameraBindGroupType,
    renderer::{
        debug_mesh::PreparedDebugMeshItem,
        mesh2d::PreparedRenderItem,
        pipelines::{Pipeline, PipelineType},
    },
};

pub(crate) fn render_2d_batch<'a>(
    sprite_batch: &'a Vec<PreparedRenderItem>,
    render_pass: &mut RenderPass<'a>,
    pipelines: &'a HashMap<PipelineType, Pipeline>,
    camera_bind_group: &'a HashMap<CameraBindGroupType, Arc<wgpu::BindGroup>>,
) {
    let pipeline = pipelines.get(&PipelineType::Mesh2d).unwrap();

    render_pass.set_pipeline(&pipeline.render_pipeline);

    render_pass.set_bind_group(
        0,
        camera_bind_group
            .get(&CameraBindGroupType::Orthographic)
            .unwrap(),
        &[],
    );

    for batch in sprite_batch {
        render_pass.set_bind_group(1, &batch.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
        render_pass.set_index_buffer(batch.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..batch.indices_len, 0, 0..1);
    }
}

pub(crate) fn render_debug_meshes<'a>(
    meshes: &'a Vec<PreparedDebugMeshItem>,
    render_pass: &mut RenderPass<'a>,
    pipelines: &'a HashMap<PipelineType, Pipeline>,
    camera_bind_group: &'a HashMap<CameraBindGroupType, Arc<wgpu::BindGroup>>,
) {
    let pipeline = pipelines.get(&PipelineType::DebugMesh).unwrap();

    render_pass.set_pipeline(&pipeline.render_pipeline);

    render_pass.set_bind_group(
        0,
        camera_bind_group
            .get(&CameraBindGroupType::Perspective)
            .unwrap(),
        &[],
    );

    for mesh in meshes {
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.draw(0..mesh.vertices_len, 0..1)
    }
}

pub(crate) fn render_ui_batch<'a>(
    sprite_batch: &'a Vec<PreparedRenderItem>,
    render_pass: &mut RenderPass<'a>,
    pipelines: &'a HashMap<PipelineType, Pipeline>,
    camera_bind_group: &'a HashMap<CameraBindGroupType, Arc<wgpu::BindGroup>>,
) {
    let pipeline = pipelines.get(&PipelineType::Mesh2d).unwrap();

    render_pass.set_pipeline(&pipeline.render_pipeline);
    render_pass.set_bind_group(
        0,
        camera_bind_group
            .get(&CameraBindGroupType::OrthographicUI)
            .unwrap(),
        &[],
    );

    for batch in sprite_batch {
        render_pass.set_bind_group(1, &batch.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
        render_pass.set_index_buffer(batch.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..batch.indices_len, 0, 0..1);
    }
}
