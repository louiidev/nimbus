use crate::{arena::ArenaId, shader::Shader, Texture};

#[derive(Debug, Clone)]
pub struct Material {
    pub texture: ArenaId<Texture>,
    normals_texture: Option<ArenaId<Texture>>,
    name: String,
    shader: ArenaId<Shader>,
}

// #[derive(Debug)]
// pub struct Material2D {
//     texture: ArenaId<Texture>,
//     name: String,
//     shader: ArenaId<Shader>,
// }

// impl Default for Material2D {
//     fn default() -> Self {
//         Self {
//             texture: ArenaId::first(),
//             name: "Default 2D Material".to_string(),
//             shader: ArenaId::first(),
//         }
//     }
// }

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
