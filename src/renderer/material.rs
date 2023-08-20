use crate::{arena::ArenaId, shader::Shader, Texture};

#[derive(Debug, Clone)]
pub struct Material {
    pub texture: ArenaId<Texture>,
    normals_texture: Option<ArenaId<Texture>>,
    name: String,
    pub shader: ArenaId<Shader>,
}

impl Material {
    pub fn new(shader: ArenaId<Shader>) -> Self {
        Material {
            shader,
            ..Default::default()
        }
    }

    pub fn with_texture(mut self, texture: ArenaId<Texture>) -> Self {
        self.texture = texture;

        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            texture: ArenaId::first(),
            normals_texture: None,
            name: "Default 3D Material".to_string(),
            shader: ArenaId::first(),
        }
    }
}
