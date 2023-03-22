use glam::Vec2;

use crate::components::{color::Color, rect::Rect, sprite::Sprite, transform::Transform};

use super::{
    debug_mesh::DebugMesh,
    mesh2d::{Mesh2d, Vertex2D, QUAD_INDICES, QUAD_UVS, QUAD_VERTEX_POSITIONS},
    Renderer,
};

impl Renderer {
    pub fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform) {
        let mut uvs = QUAD_UVS;
        if sprite.flip_x {
            uvs = [uvs[1], uvs[0], uvs[3], uvs[2]];
        }
        if sprite.flip_y {
            uvs = [uvs[3], uvs[2], uvs[1], uvs[0]];
        }

        let mut vertices = Vec::new();

        let texture = self.textures.get(sprite.texture_id).unwrap();
        let current_image_size = texture.dimensions.as_vec2();

        // By default, the size of the quad is the size of the texture
        let mut quad_size = current_image_size;

        // If a rect is specified, adjust UVs and the size of the quad
        if let Some(rect) = sprite.texture_rect {
            let rect_size = rect.size();
            for uv in &mut uvs {
                *uv = (rect.min + *uv * rect_size) / current_image_size;
            }
            quad_size = rect_size;
        }

        // Override the size if a custom one is specified
        if let Some(custom_size) = sprite.custom_size {
            quad_size = custom_size;
        }

        let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
            transform
                .transform_point(((quad_pos - sprite.anchor.as_vec()) * quad_size).extend(0.))
                .into()
        });

        for i in 0..QUAD_VERTEX_POSITIONS.len() {
            vertices.push(Vertex2D {
                position: positions[i],
                uv: uvs[i].into(),
                color: sprite.color.into(),
            });
        }

        let mesh = Mesh2d::new(sprite.texture_id, vertices, QUAD_INDICES.to_vec());

        self.render_batch_2d.push((mesh, transform.translation));
    }

    pub fn draw_debug_rect(&mut self, rect: &Rect, color: Color) {
        self.render_batch_debug.push(DebugMesh::rect(rect, color));
    }

    pub fn draw_debug_line(&mut self, p1: Vec2, p2: Vec2, color: Color) {
        self.render_batch_debug.push(DebugMesh::line(p1, p2, color));
    }
}
