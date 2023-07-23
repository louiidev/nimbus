use super::{
    material::Material,
    mesh::{AttributeValue, Mesh, MeshAttribute, MeshBuilder},
    rect::Rect,
};
use crate::{arena::ArenaId, components::color::Color};
use glam::Vec3;
use wgpu::include_wgsl;

#[derive(Debug)]
pub struct LineMaterial;

impl Material for LineMaterial {
    fn shader(&self) -> wgpu::ShaderModuleDescriptor {
        include_wgsl!("./default_shaders/line.wgsl")
    }

    fn vertex_attributes(&self) -> std::collections::BTreeSet<MeshAttribute> {
        std::collections::BTreeSet::from([MeshAttribute::Position, MeshAttribute::Color])
    }

    fn topology(&self) -> wgpu::PrimitiveTopology {
        wgpu::PrimitiveTopology::LineStrip
    }

    fn has_texture(&self) -> bool {
        false
    }

    fn filterable_texture(&self) -> bool {
        false
    }

    fn label(&self) -> &str {
        "Line Material"
    }
}

pub struct LineMeshBuilder {
    mesh_builder: MeshBuilder,
}

impl LineMeshBuilder {
    pub fn new() -> Self {
        let mb = MeshBuilder::new();

        Self {
            mesh_builder: mb.with_material(ArenaId::second()).with_batch(false),
        }
    }

    pub fn line(self, a: Vec3, b: Vec3, color: &Color) -> Mesh {
        self.mesh_builder
            .with_attributes(
                MeshAttribute::Position,
                vec![
                    AttributeValue::Position(a.into()),
                    AttributeValue::Position(b.into()),
                ],
            )
            .with_attribute(
                MeshAttribute::Color,
                super::mesh::AttributeValue::Color(color.as_rgba_f32()),
            )
            .build()
    }

    pub fn rect(self, rect: &Rect, color: &Color) -> Mesh {
        self.mesh_builder
            .with_attributes(
                MeshAttribute::Position,
                vec![
                    AttributeValue::Position(rect.min.extend(1.).into()),
                    AttributeValue::Position(Vec3::new(rect.max.x, rect.min.y, 0.).into()),
                    AttributeValue::Position(rect.max.extend(1.).into()),
                    AttributeValue::Position(Vec3::new(rect.min.x, rect.max.y, 0.).into()),
                    AttributeValue::Position(rect.min.extend(1.).into()),
                ],
            )
            .with_attribute(
                MeshAttribute::Color,
                super::mesh::AttributeValue::Color(color.as_rgba_f32()),
            )
            .build()
    }
}
