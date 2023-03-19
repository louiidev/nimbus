use wgpu::{BindGroupLayout, RenderPipeline};

pub struct Pipeline {
    pub render_pipeline: RenderPipeline,
    pub texture_bind_group_layout: BindGroupLayout,
    pub camera_bind_group_layout: BindGroupLayout,
}

#[derive(PartialEq, Eq, Hash)]
pub enum PipelineType {
    Mesh2d,
    Mesh3d,
    DebugMesh,
}
