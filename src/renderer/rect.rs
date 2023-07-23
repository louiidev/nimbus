use std::{collections::BTreeMap, ops::Add};

use glam::Vec2;

use crate::{
    components::color::Color,
    mesh::{
        AttributeValue, Mesh, MeshAttribute, Vertex, QUAD_INDICES, QUAD_UVS, QUAD_VERTEX_POSITIONS,
    },
    Anchor, ArenaId, Transform,
};

#[derive(Default, Clone, Copy, Debug)]
pub struct Rect {
    /// The minimum corner point of the rect.
    pub min: Vec2,
    /// The maximum corner point of the rect.
    pub max: Vec2,
}

impl Rect {
    pub fn from_center_size(origin: Vec2, size: Vec2) -> Self {
        assert!(size.cmpge(Vec2::ZERO).all());
        let half_size = size / 2.;
        Self::from_center_half_size(origin, half_size)
    }
    #[inline]
    pub fn from_center_half_size(origin: Vec2, half_size: Vec2) -> Self {
        assert!(half_size.cmpge(Vec2::ZERO).all());
        Self {
            min: origin - half_size,
            max: origin + half_size,
        }
    }

    pub fn from_corners(p0: Vec2, p1: Vec2) -> Self {
        Rect {
            min: p0.min(p1),
            max: p0.max(p1),
        }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn new(size: Vec2) -> Self {
        Self {
            min: Vec2::ZERO,
            max: size,
        }
    }

    pub fn into_mesh(&self, color: Color) -> Mesh {
        let quad_size = self.size();
        let transform = Transform::from_position(self.min.extend(0.));
        let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
            transform
                .transform_point(((quad_pos - Anchor::TopLeft.as_vec()) * quad_size).extend(0.))
                .into()
        });

        let vertices = positions
            .iter()
            .zip(QUAD_UVS)
            .map(|(position, uv)| {
                Vertex(BTreeMap::from([
                    (MeshAttribute::Position, AttributeValue::Position(*position)),
                    (MeshAttribute::UV, AttributeValue::UV(uv.into())),
                    (
                        MeshAttribute::Color,
                        AttributeValue::Color(color.as_rgba_f32()),
                    ),
                ]))
            })
            .collect();

        let material_handle = ArenaId::first();

        Mesh::new(
            Some(ArenaId::first()),
            material_handle,
            vertices,
            crate::mesh::Indices::U16(QUAD_INDICES.to_vec()),
            (transform.position).length(),
        )
    }
}

impl Add<Vec2> for Rect {
    type Output = Rect;
    fn add(self, other: Vec2) -> Self {
        Self {
            min: self.min + other,
            max: self.max + other,
        }
    }
}
