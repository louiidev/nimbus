use std::collections::HashMap;

use wgpu::{BindGroupLayout, RenderPipeline};

pub struct Pipeline {
    pub render_pipeline: RenderPipeline,
    pub bind_group_layouts: HashMap<BindGroupLayoutType, BindGroupLayout>,
}

#[derive(PartialEq, Eq, Hash)]
pub enum PipelineType {
    Mesh2d,
    _Mesh3d,
    DebugMesh,
}

#[derive(PartialEq, Eq, Hash)]
pub enum BindGroupLayoutType {
    Camera,
    Texture,
}
