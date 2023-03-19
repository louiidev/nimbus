use std::{collections::HashMap, sync::Arc};

use wgpu::RenderPass;

use crate::{
    camera::CameraBindGroupType,
    renderer::{
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
