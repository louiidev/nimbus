use std::{collections::BTreeSet, fmt::Debug};

use wgpu::{
    include_wgsl, BindGroup, BindGroupLayout, Device, PrimitiveTopology, RenderPipeline,
    ShaderModuleDescriptor,
};

use crate::arena::ArenaId;

use super::{
    mesh::{Mesh, MeshAttribute},
    pipeline::Pipeline,
    Renderer,
};

pub type MaterialHandle = ArenaId<Pipeline>;

#[derive(Debug)]
pub struct DefaultMat {}
impl Material for DefaultMat {}

pub trait Material: Debug {
    fn shader(&self) -> ShaderModuleDescriptor {
        include_wgsl!("./default_shaders/default.wgsl")
    }

    fn vertex_attributes(&self) -> BTreeSet<MeshAttribute> {
        BTreeSet::from([
            MeshAttribute::Position,
            MeshAttribute::UV,
            MeshAttribute::Color,
        ])
    }

    fn get_bind_group_layouts(&self, _device: &Device) -> Vec<BindGroupLayout> {
        Vec::default()
    }

    fn get_bind_groups(
        &self,
        mesh: &Mesh,
        renderer: &Renderer,
        render_pipeline: &RenderPipeline,
    ) -> Vec<BindGroup> {
        Vec::default()
    }

    fn topology(&self) -> PrimitiveTopology {
        PrimitiveTopology::TriangleList
    }

    fn has_texture(&self) -> bool {
        true
    }

    fn use_depth_stencil(&self) -> bool {
        false
    }

    fn filterable_texture(&self) -> bool {
        true
    }

    fn label(&self) -> &str {
        "Default Material"
    }
}

impl Renderer {
    pub fn push_material(&mut self, material: impl Material + 'static) -> ArenaId<Pipeline> {
        let pipeline = Pipeline {
            render_pipeline: self.create_pipeline_from_material(&material),
            material: Box::from(material),
        };

        self.materials.insert(pipeline)
    }
}
