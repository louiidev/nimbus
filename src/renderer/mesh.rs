use std::collections::BTreeMap;
use std::sync::Arc;

use bevy_ecs::prelude::Component;
use bytemuck::cast_slice;
use uuid::Uuid;
use wgpu::BufferUsages;
use wgpu::Device;
use wgpu::IndexFormat;
use wgpu::PrimitiveTopology;
use wgpu::RenderPipeline;
use wgpu::TextureFormat;
use wgpu::VertexFormat;

use glam::Vec2;
use glam::Vec3;
use wgpu::include_wgsl;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

macro_rules! impl_from {
    ($from:ty, $variant:tt) => {
        impl From<Vec<$from>> for VertexAttributeValues {
            fn from(vec: Vec<$from>) -> Self {
                VertexAttributeValues::$variant(vec)
            }
        }
    };
}

macro_rules! impl_from_into {
    ($from:ty, $variant:tt) => {
        impl From<Vec<$from>> for VertexAttributeValues {
            fn from(vec: Vec<$from>) -> Self {
                let vec: Vec<_> = vec.into_iter().map(|t| t.into()).collect();
                VertexAttributeValues::$variant(vec)
            }
        }
    };
}

impl_from!(f32, Float32);
impl_from!([f32; 2], Float32x2);
impl_from_into!(Vec2, Float32x2);
impl_from!([f32; 3], Float32x3);
impl_from_into!(Vec3, Float32x3);
impl_from!([f32; 4], Float32x4);

pub trait VertexFormatSize {
    fn get_size(self) -> u64;
}

impl VertexFormatSize for wgpu::VertexFormat {
    #[allow(clippy::match_same_arms)]
    fn get_size(self) -> u64 {
        match self {
            VertexFormat::Uint8x2 => 2,
            VertexFormat::Uint8x4 => 4,
            VertexFormat::Sint8x2 => 2,
            VertexFormat::Sint8x4 => 4,
            VertexFormat::Unorm8x2 => 2,
            VertexFormat::Unorm8x4 => 4,
            VertexFormat::Snorm8x2 => 2,
            VertexFormat::Snorm8x4 => 4,
            VertexFormat::Uint16x2 => 2 * 2,
            VertexFormat::Uint16x4 => 2 * 4,
            VertexFormat::Sint16x2 => 2 * 2,
            VertexFormat::Sint16x4 => 2 * 4,
            VertexFormat::Unorm16x2 => 2 * 2,
            VertexFormat::Unorm16x4 => 2 * 4,
            VertexFormat::Snorm16x2 => 2 * 2,
            VertexFormat::Snorm16x4 => 2 * 4,
            VertexFormat::Float16x2 => 2 * 2,
            VertexFormat::Float16x4 => 2 * 4,
            VertexFormat::Float32 => 4,
            VertexFormat::Float32x2 => 4 * 2,
            VertexFormat::Float32x3 => 4 * 3,
            VertexFormat::Float32x4 => 4 * 4,
            VertexFormat::Uint32 => 4,
            VertexFormat::Uint32x2 => 4 * 2,
            VertexFormat::Uint32x3 => 4 * 3,
            VertexFormat::Uint32x4 => 4 * 4,
            VertexFormat::Sint32 => 4,
            VertexFormat::Sint32x2 => 4 * 2,
            VertexFormat::Sint32x3 => 4 * 3,
            VertexFormat::Sint32x4 => 4 * 4,
            VertexFormat::Float64 => 8,
            VertexFormat::Float64x2 => 8 * 2,
            VertexFormat::Float64x3 => 8 * 3,
            VertexFormat::Float64x4 => 8 * 4,
        }
    }
}

impl VertexAttributeValues {
    pub fn len(&self) -> usize {
        match self {
            VertexAttributeValues::Float32(values) => values.len(),
            VertexAttributeValues::Sint32(values) => values.len(),
            VertexAttributeValues::Uint32(values) => values.len(),
            VertexAttributeValues::Float32x2(values) => values.len(),
            VertexAttributeValues::Sint32x2(values) => values.len(),
            VertexAttributeValues::Uint32x2(values) => values.len(),
            VertexAttributeValues::Float32x3(values) => values.len(),
            VertexAttributeValues::Sint32x3(values) => values.len(),
            VertexAttributeValues::Uint32x3(values) => values.len(),
            VertexAttributeValues::Float32x4(values) => values.len(),
            VertexAttributeValues::Sint32x4(values) => values.len(),
            VertexAttributeValues::Uint32x4(values) => values.len(),
            VertexAttributeValues::Sint16x2(values) => values.len(),
            VertexAttributeValues::Snorm16x2(values) => values.len(),
            VertexAttributeValues::Uint16x2(values) => values.len(),
            VertexAttributeValues::Unorm16x2(values) => values.len(),
            VertexAttributeValues::Sint16x4(values) => values.len(),
            VertexAttributeValues::Snorm16x4(values) => values.len(),
            VertexAttributeValues::Uint16x4(values) => values.len(),
            VertexAttributeValues::Unorm16x4(values) => values.len(),
            VertexAttributeValues::Sint8x2(values) => values.len(),
            VertexAttributeValues::Snorm8x2(values) => values.len(),
            VertexAttributeValues::Uint8x2(values) => values.len(),
            VertexAttributeValues::Unorm8x2(values) => values.len(),
            VertexAttributeValues::Sint8x4(values) => values.len(),
            VertexAttributeValues::Snorm8x4(values) => values.len(),
            VertexAttributeValues::Uint8x4(values) => values.len(),
            VertexAttributeValues::Unorm8x4(values) => values.len(),
        }
    }

    pub fn get_bytes(&self) -> &[u8] {
        match self {
            VertexAttributeValues::Float32(values) => cast_slice(values),
            VertexAttributeValues::Sint32(values) => cast_slice(values),
            VertexAttributeValues::Uint32(values) => cast_slice(values),
            VertexAttributeValues::Float32x2(values) => cast_slice(values),
            VertexAttributeValues::Sint32x2(values) => cast_slice(values),
            VertexAttributeValues::Uint32x2(values) => cast_slice(values),
            VertexAttributeValues::Float32x3(values) => cast_slice(values),
            VertexAttributeValues::Sint32x3(values) => cast_slice(values),
            VertexAttributeValues::Uint32x3(values) => cast_slice(values),
            VertexAttributeValues::Float32x4(values) => cast_slice(values),
            VertexAttributeValues::Sint32x4(values) => cast_slice(values),
            VertexAttributeValues::Uint32x4(values) => cast_slice(values),
            VertexAttributeValues::Sint16x2(values) => cast_slice(values),
            VertexAttributeValues::Snorm16x2(values) => cast_slice(values),
            VertexAttributeValues::Uint16x2(values) => cast_slice(values),
            VertexAttributeValues::Unorm16x2(values) => cast_slice(values),
            VertexAttributeValues::Sint16x4(values) => cast_slice(values),
            VertexAttributeValues::Snorm16x4(values) => cast_slice(values),
            VertexAttributeValues::Uint16x4(values) => cast_slice(values),
            VertexAttributeValues::Unorm16x4(values) => cast_slice(values),
            VertexAttributeValues::Sint8x2(values) => cast_slice(values),
            VertexAttributeValues::Snorm8x2(values) => cast_slice(values),
            VertexAttributeValues::Uint8x2(values) => cast_slice(values),
            VertexAttributeValues::Unorm8x2(values) => cast_slice(values),
            VertexAttributeValues::Sint8x4(values) => cast_slice(values),
            VertexAttributeValues::Snorm8x4(values) => cast_slice(values),
            VertexAttributeValues::Uint8x4(values) => cast_slice(values),
            VertexAttributeValues::Unorm8x4(values) => cast_slice(values),
        }
    }
}

impl From<&VertexAttributeValues> for VertexFormat {
    fn from(values: &VertexAttributeValues) -> Self {
        match values {
            VertexAttributeValues::Float32(_) => VertexFormat::Float32,
            VertexAttributeValues::Sint32(_) => VertexFormat::Sint32,
            VertexAttributeValues::Uint32(_) => VertexFormat::Uint32,
            VertexAttributeValues::Float32x2(_) => VertexFormat::Float32x2,
            VertexAttributeValues::Sint32x2(_) => VertexFormat::Sint32x2,
            VertexAttributeValues::Uint32x2(_) => VertexFormat::Uint32x2,
            VertexAttributeValues::Float32x3(_) => VertexFormat::Float32x3,
            VertexAttributeValues::Sint32x3(_) => VertexFormat::Sint32x3,
            VertexAttributeValues::Uint32x3(_) => VertexFormat::Uint32x3,
            VertexAttributeValues::Float32x4(_) => VertexFormat::Float32x4,
            VertexAttributeValues::Sint32x4(_) => VertexFormat::Sint32x4,
            VertexAttributeValues::Uint32x4(_) => VertexFormat::Uint32x4,
            VertexAttributeValues::Sint16x2(_) => VertexFormat::Sint16x2,
            VertexAttributeValues::Snorm16x2(_) => VertexFormat::Snorm16x2,
            VertexAttributeValues::Uint16x2(_) => VertexFormat::Uint16x2,
            VertexAttributeValues::Unorm16x2(_) => VertexFormat::Unorm16x2,
            VertexAttributeValues::Sint16x4(_) => VertexFormat::Sint16x4,
            VertexAttributeValues::Snorm16x4(_) => VertexFormat::Snorm16x4,
            VertexAttributeValues::Uint16x4(_) => VertexFormat::Uint16x4,
            VertexAttributeValues::Unorm16x4(_) => VertexFormat::Unorm16x4,
            VertexAttributeValues::Sint8x2(_) => VertexFormat::Sint8x2,
            VertexAttributeValues::Snorm8x2(_) => VertexFormat::Snorm8x2,
            VertexAttributeValues::Uint8x2(_) => VertexFormat::Uint8x2,
            VertexAttributeValues::Unorm8x2(_) => VertexFormat::Unorm8x2,
            VertexAttributeValues::Sint8x4(_) => VertexFormat::Sint8x4,
            VertexAttributeValues::Snorm8x4(_) => VertexFormat::Snorm8x4,
            VertexAttributeValues::Uint8x4(_) => VertexFormat::Uint8x4,
            VertexAttributeValues::Unorm8x4(_) => VertexFormat::Unorm8x4,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MeshVertexAttributeId(usize);

#[derive(Debug, Clone)]
pub struct MeshVertexAttribute {
    /// The friendly name of the vertex attribute
    pub name: &'static str,

    /// The _unique_ id of the vertex attribute. This will also determine sort ordering
    /// when generating vertex buffers. Built-in / standard attributes will use "close to zero"
    /// indices. When in doubt, use a random / very large usize to avoid conflicts.
    pub id: MeshVertexAttributeId,

    /// The format of the vertex attribute.
    pub format: VertexFormat,
}

impl MeshVertexAttribute {
    pub const fn new(name: &'static str, id: usize, format: VertexFormat) -> Self {
        Self {
            name,
            id: MeshVertexAttributeId(id),
            format,
        }
    }
}

pub struct MeshAttributeData {
    attribute: MeshVertexAttribute,
    values: VertexAttributeValues,
}
pub enum Indices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

/// An Iterator for the [`Indices`].
enum IndicesIter<'a> {
    U16(std::slice::Iter<'a, u16>),
    U32(std::slice::Iter<'a, u32>),
}

impl Indices {
    /// Returns the number of indices.
    pub fn len(&self) -> usize {
        match self {
            Indices::U16(vec) => vec.len(),
            Indices::U32(vec) => vec.len(),
        }
    }
}

impl From<&Indices> for IndexFormat {
    fn from(indices: &Indices) -> Self {
        match indices {
            Indices::U16(_) => IndexFormat::Uint16,
            Indices::U32(_) => IndexFormat::Uint32,
        }
    }
}

/// Contains an array where each entry describes a property of a single vertex.
/// Matches the [`VertexFormats`](VertexFormat).
#[derive(Clone, Debug)]
pub enum VertexAttributeValues {
    Float32(Vec<f32>),
    Sint32(Vec<i32>),
    Uint32(Vec<u32>),
    Float32x2(Vec<[f32; 2]>),
    Sint32x2(Vec<[i32; 2]>),
    Uint32x2(Vec<[u32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Sint32x3(Vec<[i32; 3]>),
    Uint32x3(Vec<[u32; 3]>),
    Float32x4(Vec<[f32; 4]>),
    Sint32x4(Vec<[i32; 4]>),
    Uint32x4(Vec<[u32; 4]>),
    Sint16x2(Vec<[i16; 2]>),
    Snorm16x2(Vec<[i16; 2]>),
    Uint16x2(Vec<[u16; 2]>),
    Unorm16x2(Vec<[u16; 2]>),
    Sint16x4(Vec<[i16; 4]>),
    Snorm16x4(Vec<[i16; 4]>),
    Uint16x4(Vec<[u16; 4]>),
    Unorm16x4(Vec<[u16; 4]>),
    Sint8x2(Vec<[i8; 2]>),
    Snorm8x2(Vec<[i8; 2]>),
    Uint8x2(Vec<[u8; 2]>),
    Unorm8x2(Vec<[u8; 2]>),
    Sint8x4(Vec<[i8; 4]>),
    Snorm8x4(Vec<[i8; 4]>),
    Uint8x4(Vec<[u8; 4]>),
    Unorm8x4(Vec<[u8; 4]>),
}

#[derive(Clone, Debug)]
pub struct Buffer {
    id: Uuid,
    pub value: Arc<wgpu::Buffer>,
}

impl Buffer {
    pub fn new(buffer: wgpu::Buffer) -> Self {
        Self {
            id: Uuid::new_v4(),
            value: Arc::new(buffer),
        }
    }
}

/// The index/vertex buffer info of a [`GpuMesh`].
#[derive(Debug, Clone)]
pub enum GpuBufferInfo {
    Indexed {
        /// Contains all index data of a mesh.
        buffer: Buffer,
        count: u32,
        index_format: IndexFormat,
    },
    NonIndexed {
        vertex_count: u32,
    },
}

pub struct GpuMesh {
    /// Contains all attribute data for each vertex.
    pub vertex_buffer: Buffer,
    pub buffer_info: GpuBufferInfo,
}

#[derive(Component, Default)]
pub struct Mesh {
    pub id: Uuid,
    pub attributes: BTreeMap<MeshVertexAttributeId, MeshAttributeData>,
    pub indices: Option<Indices>,
}

impl Mesh {
    pub const ATTRIBUTE_POSITION: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x3);

    pub const ATTRIBUTE_NORMAL: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Normal", 1, VertexFormat::Float32x3);

    pub const ATTRIBUTE_UV_0: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Uv", 2, VertexFormat::Float32x2);

    /// The direction of the vertex tangent. Used for normal mapping
    pub const ATTRIBUTE_TANGENT: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Tangent", 3, VertexFormat::Float32x4);

    /// Per vertex coloring. Use in conjunction with [`Mesh::insert_attribute`]
    pub const ATTRIBUTE_COLOR: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Color", 4, VertexFormat::Float32x4);

    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            attributes: BTreeMap::new(),
            indices: None,
        }
    }

    pub fn get_attribute_array_stride() -> u64 {
        let positions_stride = Mesh::ATTRIBUTE_POSITION.format.size();
        let color_stride = Mesh::ATTRIBUTE_UV_0.format.size();
        positions_stride + color_stride
    }

    pub fn count_vertices(&self) -> usize {
        let mut vertex_count: Option<usize> = None;
        for (attribute_id, attribute_data) in &self.attributes {
            let attribute_len = attribute_data.values.len();
            if let Some(previous_vertex_count) = vertex_count {
                assert_eq!(previous_vertex_count, attribute_len,
                        "{attribute_id:?} has a different vertex count ({attribute_len}) than other attributes ({previous_vertex_count}) in this mesh.");
            }
            vertex_count = Some(attribute_len);
        }

        vertex_count.unwrap_or(0)
    }

    pub fn get_vertex_buffer_data(&self) -> Vec<u8> {
        let mut vertex_size = 0;
        for attribute_data in self.attributes.values() {
            let vertex_format = attribute_data.attribute.format;
            vertex_size += vertex_format.get_size() as usize;
        }

        let vertex_count = self.count_vertices();
        let mut attributes_interleaved_buffer = vec![0; vertex_count * vertex_size];
        // bundle into interleaved buffers
        let mut attribute_offset = 0;
        for attribute_data in self.attributes.values() {
            let attribute_size = attribute_data.attribute.format.get_size() as usize;
            let attributes_bytes = attribute_data.values.get_bytes();
            for (vertex_index, attribute_bytes) in
                attributes_bytes.chunks_exact(attribute_size).enumerate()
            {
                let offset = vertex_index * vertex_size + attribute_offset;
                attributes_interleaved_buffer[offset..offset + attribute_size]
                    .copy_from_slice(attribute_bytes);
            }

            attribute_offset += attribute_size;
        }

        attributes_interleaved_buffer
    }

    /// Retrieves the vertex `indices` of the mesh.
    #[inline]
    pub fn indices(&self) -> Option<&Indices> {
        self.indices.as_ref()
    }

    /// Retrieves the vertex `indices` of the mesh mutably.
    #[inline]
    pub fn indices_mut(&mut self) -> Option<&mut Indices> {
        self.indices.as_mut()
    }

    /// Computes and returns the index data of the mesh as bytes.
    /// This is used to transform the index data into a GPU friendly format.
    pub fn get_index_buffer_bytes(&self) -> Option<&[u8]> {
        self.indices.as_ref().map(|indices| match &indices {
            Indices::U16(indices) => cast_slice(&indices[..]),
            Indices::U32(indices) => cast_slice(&indices[..]),
        })
    }

    pub fn insert_attribute(
        &mut self,
        attribute: MeshVertexAttribute,
        values: impl Into<VertexAttributeValues>,
    ) {
        let values = values.into();
        let values_format = VertexFormat::from(&values);
        if values_format != attribute.format {
            panic!(
                "Failed to insert attribute. Invalid attribute format for {}. Given format is {values_format:?} but expected {:?}",
                attribute.name, attribute.format
            );
        }

        self.attributes
            .insert(attribute.id, MeshAttributeData { attribute, values });
    }

    pub fn get_gpu_data(&self, render_device: &Device) -> GpuMesh {
        let vertex_buffer_data = self.get_vertex_buffer_data();
        let vertex_buffer = render_device.create_buffer_init(&BufferInitDescriptor {
            usage: BufferUsages::VERTEX,
            label: Some("Mesh Vertex Buffer"),
            contents: &vertex_buffer_data,
        });

        GpuMesh {
            vertex_buffer: Buffer::new(vertex_buffer),

            buffer_info: self.get_index_buffer_bytes().map_or(
                GpuBufferInfo::NonIndexed {
                    vertex_count: self.count_vertices() as u32,
                },
                |data| GpuBufferInfo::Indexed {
                    buffer: Buffer::new(render_device.create_buffer_init(&BufferInitDescriptor {
                        usage: BufferUsages::INDEX,
                        contents: data,
                        label: Some("Mesh Index Buffer"),
                    })),
                    count: self.indices().unwrap().len() as u32,
                    index_format: self.indices().unwrap().into(),
                },
            ),
        }
    }
}
