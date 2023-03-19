use glam::Vec3;

use crate::components::{color::Color, rect::Rect};

use super::mesh2d::QUAD_INDICES;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct DebugMeshVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl DebugMeshVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<DebugMeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>()) as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct DebugMesh {
    pub(crate) vertices: Vec<DebugMeshVertex>,
    pub(crate) indices: Vec<u16>,
}

impl From<Rect> for DebugMesh {
    fn from(rect: Rect) -> Self {
        let color = Color::GREEN.as_rgba_f32();

        // pub const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
        //     Vec2::new(-0.5, -0.5),
        //     Vec2::new(0.5, -0.5),
        //     Vec2::new(0.5, 0.5),
        //     Vec2::new(-0.5, 0.5),
        // ];

        let verts = vec![
            DebugMeshVertex {
                position: rect.min.extend(1.).into(),
                color,
            },
            DebugMeshVertex {
                position: Vec3::new(rect.max.x, rect.min.y, 0.).into(),
                color,
            },
            DebugMeshVertex {
                position: rect.max.extend(1.).into(),
                color,
            },
            DebugMeshVertex {
                position: Vec3::new(rect.min.x, rect.max.y, 0.).into(),
                color,
            },
            DebugMeshVertex {
                position: rect.min.extend(1.).into(),
                color,
            },
        ];

        DebugMesh {
            vertices: verts,
            indices: QUAD_INDICES.to_vec(),
        }
    }
}

pub struct PreparedDebugMeshItem {}
