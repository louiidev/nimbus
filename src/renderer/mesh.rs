use std::{collections::BTreeMap, mem};

use bytemuck::{cast_slice, AnyBitPattern};

use glam::Vec2;
use wgpu::{VertexAttribute, VertexFormat};

use crate::{material::Material, transform::Transform};

pub const QUAD_INDICES: [u16; 6] = [0, 2, 3, 0, 1, 2];

pub const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
    Vec2::new(-0.5, -0.5),
    Vec2::new(0.5, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

pub const QUAD_UVS: [Vec2; 4] = [
    Vec2::new(0., 1.),
    Vec2::new(1., 1.),
    Vec2::new(1., 0.),
    Vec2::new(0., 0.),
];

#[derive(Debug, Clone)]
pub enum Indices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl Indices {
    pub fn wgpu_index_format(&self) -> wgpu::IndexFormat {
        match self {
            Indices::U16(_) => wgpu::IndexFormat::Uint16,
            Indices::U32(_) => wgpu::IndexFormat::Uint32,
        }
    }

    pub fn add(&self, value: usize) -> Indices {
        match self {
            Indices::U16(indices) => {
                return Indices::U16(
                    indices
                        .iter()
                        .map(|index| index + value as u16)
                        .collect::<Vec<u16>>(),
                )
            }
            Indices::U32(indices) => {
                return Indices::U32(
                    indices
                        .iter()
                        .map(|index| index + value as u32)
                        .collect::<Vec<u32>>(),
                )
            }
        }
    }

    pub fn cast_slice<B: AnyBitPattern>(&self) -> &[B] {
        match self {
            Indices::U16(a) => bytemuck::cast_slice(a),
            Indices::U32(a) => bytemuck::cast_slice(a),
        }
    }

    pub fn append(&mut self, other: &mut Self) {
        match self {
            Indices::U16(v) => match other {
                Indices::U16(v2) => v.append(v2),
                Indices::U32(_) => panic!("Trying to append a u32 to u16"),
            },
            Indices::U32(v) => match other {
                Indices::U32(v2) => v.append(v2),
                Indices::U16(_) => panic!("Trying to append a u16 to u32"),
            },
        }
    }

    /// Returns an iterator over the indices.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        match self {
            Indices::U16(vec) => IndicesIter::U16(vec.iter()),
            Indices::U32(vec) => IndicesIter::U32(vec.iter()),
        }
    }

    /// Returns the number of indices.
    pub fn len(&self) -> usize {
        match self {
            Indices::U16(vec) => vec.len(),
            Indices::U32(vec) => vec.len(),
        }
    }

    /// Returns `true` if there are no indices.
    pub fn is_empty(&self) -> bool {
        match self {
            Indices::U16(vec) => vec.is_empty(),
            Indices::U32(vec) => vec.is_empty(),
        }
    }
}

/// An Iterator for the [`Indices`].
enum IndicesIter<'a> {
    U16(std::slice::Iter<'a, u16>),
    U32(std::slice::Iter<'a, u32>),
}

impl Iterator for IndicesIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IndicesIter::U16(iter) => iter.next().map(|val| *val as usize),
            IndicesIter::U32(iter) => iter.next().map(|val| *val as usize),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            IndicesIter::U16(iter) => iter.size_hint(),
            IndicesIter::U32(iter) => iter.size_hint(),
        }
    }
}

impl Default for Indices {
    fn default() -> Self {
        Indices::U16(Vec::default())
    }
}

pub struct MeshBuilder {
    pub(crate) material: Material,
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Indices,
}

impl MeshBuilder {
    pub fn new(material: Material) -> Self {
        Self {
            material,
            vertices: Vec::default(),
            indices: Indices::default(),
        }
    }

    pub fn quad(size: Vec2, transform: Transform) -> Self {
        MeshBuilder::new(Material::default())
            .with_attributes(
                MeshAttribute::Position,
                QUAD_VERTEX_POSITIONS
                    .iter()
                    .map(|v| {
                        AttributeValue::Position(
                            transform.transform_point((*v * size).extend(0.)).into(),
                        )
                    })
                    .collect(),
            )
            .with_indices(Indices::U16(QUAD_INDICES.to_vec()))
    }

    pub fn with_vertices(mut self, vertices: Vec<Vertex>) -> Self {
        self.vertices = vertices;

        self
    }

    pub fn with_attributes(
        mut self,
        attribute: MeshAttribute,
        values: Vec<AttributeValue>,
    ) -> Self {
        if self.vertices.is_empty() {
            self.vertices = values
                .iter()
                .map(|v| Vertex::new().with_attribute(attribute, *v))
                .collect();
        } else {
            for i in 0..values.len() {
                let vert = &mut self.vertices[i];
                vert.set_attribute(attribute, values[i]);
            }
        }

        self
    }

    pub fn with_attribute(mut self, attribute: MeshAttribute, value: AttributeValue) -> Self {
        for i in 0..self.vertices.len() {
            let vert = &mut self.vertices[i];
            vert.set_attribute(attribute, value);
        }

        self
    }

    pub fn with_indices(mut self, indices: Indices) -> Self {
        self.indices = indices;

        self
    }

    pub fn build(self) -> Mesh {
        Mesh {
            material: self.material,
            vertices: self.vertices,
            indices: self.indices,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub material: Material,
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Indices,
}
impl Mesh {
    pub fn new(material: Material, vertices: Vec<Vertex>, indices: Indices) -> Self {
        Self {
            material,
            vertices,
            indices,
        }
    }

    pub fn concat(&mut self, mut vertices: Vec<Vertex>, mut indices: Indices) {
        self.vertices.append(&mut vertices);
        self.indices.append(&mut indices);
    }
}

#[derive(Hash, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MeshAttribute {
    Position,
    UV,
    Color,
    Normal,
}

impl MeshAttribute {
    pub fn size(&self) -> usize {
        match self {
            MeshAttribute::Position => mem::size_of::<[f32; 3]>(),
            MeshAttribute::UV => mem::size_of::<[f32; 2]>(),
            MeshAttribute::Color => mem::size_of::<[f32; 4]>(),
            MeshAttribute::Normal => mem::size_of::<[f32; 3]>(),
        }
    }

    fn format(&self) -> wgpu::VertexFormat {
        match self {
            MeshAttribute::Position => VertexFormat::Float32x3,
            MeshAttribute::UV => VertexFormat::Float32x2,
            MeshAttribute::Color => VertexFormat::Float32x4,
            MeshAttribute::Normal => VertexFormat::Float32x3,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AttributeValue {
    Position([f32; 3]),
    UV([f32; 2]),
    Color([f32; 4]),
    Normal([f32; 3]),
}

impl AttributeValue {
    fn into_mesh_attr(&self) -> MeshAttribute {
        match self {
            AttributeValue::Position(_) => MeshAttribute::Position,
            AttributeValue::UV(_) => MeshAttribute::UV,
            AttributeValue::Color(_) => MeshAttribute::Color,
            AttributeValue::Normal(_) => MeshAttribute::Normal,
        }
    }

    fn get_bytes(&self) -> &[u8] {
        match self {
            AttributeValue::Position(values) => cast_slice(values),
            AttributeValue::UV(values) => cast_slice(values),
            AttributeValue::Color(values) => cast_slice(values),
            AttributeValue::Normal(values) => cast_slice(values),
        }
    }
}

pub fn get_attribute_layout<'a>(
    attributes: impl Iterator<Item = &'a MeshAttribute>,
) -> (Vec<VertexAttribute>, u64) {
    let mut vertex_attribute: Vec<VertexAttribute> = Vec::new();
    let mut offset: usize = 0;
    for (index, data) in attributes.enumerate() {
        vertex_attribute.push(VertexAttribute {
            format: data.format(),
            offset: offset as u64,
            shader_location: index as u32,
        });

        offset += data.size();
    }

    (vertex_attribute, offset as u64)
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertex(pub BTreeMap<MeshAttribute, AttributeValue>);

impl Vertex {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn with_attribute(mut self, attribute: MeshAttribute, value: AttributeValue) -> Self {
        self.0.insert(attribute, value);

        self
    }

    pub fn set_attribute(&mut self, attribute: MeshAttribute, value: AttributeValue) {
        self.0.insert(attribute, value);
    }

    // TODO: This seems a little slow,
    // Maybe it could be better
    pub fn get_bytes(&self) -> Vec<u8> {
        let mut base = Vec::new();

        for attr in self.0.values() {
            base.append(&mut attr.get_bytes().to_vec());
        }

        base
    }
}
