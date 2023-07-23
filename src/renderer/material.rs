use crate::{arena::ArenaId, shader::Shader, Texture};

pub struct Material3D {
    texture: ArenaId<Texture>,
    normals_texture: Option<ArenaId<Texture>>,
    name: String,
    shader: ArenaId<Shader>,
}
