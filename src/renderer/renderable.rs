use std::collections::HashMap;

use bevy_ecs::system::Resource;
use uuid::Uuid;
use wgpu::{BindGroup, Buffer, RenderPass, RenderPipeline};

pub struct Renderable {
    pipeline_id: Uuid,
    bindgroup_ids: Vec<Uuid>,
    vertex_buffer_id: Uuid,
    index_buffer_id: Option<Uuid>,
    draw_amount: u32,
}

#[derive(Default, Resource)]
pub struct RenderCache {
    pub pipelines: HashMap<Uuid, RenderPipeline>,
    pub bind_groups: HashMap<Uuid, BindGroup>,
    pub buffers: HashMap<Uuid, Buffer>,
}

pub fn render_generic<'a>(
    render_batch: &'a mut Vec<Renderable>,
    render_pass: &mut RenderPass<'a>,
    render_cache: &'a RenderCache,
) {
    for batch in render_batch.iter_mut() {
        let pipeline = render_cache.pipelines.get(&batch.pipeline_id).unwrap();

        render_pass.set_pipeline(&pipeline);
        for (index, bind_group_id) in batch.bindgroup_ids.iter().enumerate() {
            let bind_group = render_cache.bind_groups.get(&bind_group_id).unwrap();
            render_pass.set_bind_group(index as u32, &bind_group, &[]);
        }

        let vertex_buffer = render_cache.buffers.get(&batch.vertex_buffer_id).unwrap();
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

        if let Some(index_buffer_id) = batch.index_buffer_id {
            let index_buffer = render_cache.buffers.get(&index_buffer_id).unwrap();
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..batch.draw_amount, 0, 0..1);
        } else {
            render_pass.draw(0..batch.draw_amount, 0..1);
        }
    }
}
