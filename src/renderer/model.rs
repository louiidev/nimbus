use glam::{Vec2, Vec3, Vec4};

use crate::{material::Material, ArenaId, Texture};

#[derive(Debug)]
pub struct Model {
    pub material: Material,
    pub(crate) texture: ArenaId<Texture>,
    pub(crate) positions: Vec<Vec3>,
    pub(crate) tex_coords: Vec<Vec2>,
    pub(crate) normals: Vec<Vec3>,
    pub(crate) indices: Vec<u32>,
    pub(crate) colors: Vec<Vec4>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            material: Default::default(),
            texture: ArenaId::first(),
            positions: Default::default(),
            colors: Default::default(),
            tex_coords: Default::default(),
            normals: Default::default(),
            indices: Default::default(),
        }
    }
}
