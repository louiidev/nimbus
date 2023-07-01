use crate::{
    arena::ArenaId,
    components::{color::Color, rect::Rect, sprite::Sprite, text::Text, transform::Transform},
};
use glam::{Mat4, Vec2, Vec3};

use super::{
    debug_mesh::DebugMesh,
    mesh2d::{Mesh2d, Vertex2D, QUAD_INDICES, QUAD_UVS, QUAD_VERTEX_POSITIONS},
    Renderer,
};

impl Renderer {
    pub fn draw_geometry(&mut self, vertices: Vec<Vertex2D>, indices: Vec<u16>, position: Vec2) {
        let vertices: Vec<Vertex2D> = vertices
            .iter()
            .map(|vertex| Vertex2D {
                position: (Vec2::new(vertex.position[0], vertex.position[1]) + position)
                    .extend(0.)
                    .into(),
                uv: vertex.uv,
                color: vertex.color,
            })
            .collect();

        let mesh = Mesh2d::new(ArenaId::first(), vertices, indices);

        self.meshes2d.push((mesh, position.extend(0.)));
    }

    pub fn draw_rect(&mut self, rect: &Rect, color: Color) {
        let mesh = Mesh2d::rect(rect.center(), rect.size(), color);

        self.meshes2d.push((mesh, rect.center().extend(0.)));
    }

    pub fn draw_text_basic(&mut self, text: &str, position: Vec2) {
        self.draw_text(
            &text.into(),
            Transform::from_translation(Vec3::new(position.x, position.y, 0.)),
        );
    }

    pub fn draw_text(&mut self, text: &Text, mut transform: Transform) {
        let text_glyphs = self.font_renderer.queue_text(
            text,
            None,
            fontdue::layout::CoordinateSystem::PositiveYUp,
            &mut self.textures,
            &self.device,
            &self.queue,
        );

        let size = text_glyphs.iter().fold(
            Vec2::default(),
            |mut size: Vec2, text_glyph: &super::font_renderer::font::PositionedGlyph| {
                let rect = text_glyph.rect;
                let glyph_position = text_glyph.position - -Vec2::new(-0.5, -0.5);

                let x_distance = glyph_position.x - size.x;
                let actual_glyph_size = rect.size();
                size.y = size.y.max(actual_glyph_size.y);
                size.x += actual_glyph_size.x + x_distance;

                size
            },
        );

        let offset = Vec2::new(size.x / 2. * transform.scale.x, -size.y * transform.scale.y);
        transform.translation -= offset.extend(0.);
        for text_glyph in text_glyphs {
            // let transform = Transform::from_translation(position + text_glyph.position.extend(0.));
            let mut transform = transform.clone();
            transform.translation = transform.transform_point(text_glyph.position.extend(0.));
            let current_image_size = text_glyph.atlas_info.atlas_size;
            let scale_factor = 1f32;
            let rect = text_glyph.rect;

            let mut vertices = Vec::new();

            let uvs = [
                Vec2::new(rect.min.x, rect.max.y),
                Vec2::new(rect.max.x, rect.max.y),
                Vec2::new(rect.max.x, rect.min.y),
                Vec2::new(rect.min.x, rect.min.y),
            ]
            .map(|pos| pos / current_image_size);

            let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
                transform
                    .transform_point(((quad_pos - Vec2::new(-0.5, -0.5)) * rect.size()).extend(0.))
                    .into()
            });

            for i in 0..QUAD_VERTEX_POSITIONS.len() {
                vertices.push(Vertex2D {
                    position: positions[i],
                    uv: uvs[i].into(),
                    color: text.theme.color.as_rgba_f32(),
                });
            }
            let meta = Mesh2d {
                texture_id: text_glyph.atlas_info.texture_id,
                vertices,
                indices: QUAD_INDICES.to_vec(),
            };

            self.meshes2d.push((meta, transform.translation));
        }
    }

    pub fn draw_sprite_basic(&mut self, sprite: &Sprite, position: Vec3) {
        self.draw_sprite(sprite, &Transform::from_translation(position))
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform) {
        let mut uvs = QUAD_UVS;
        if sprite.flip_x {
            uvs = [uvs[1], uvs[0], uvs[3], uvs[2]];
        }
        if sprite.flip_y {
            uvs = [uvs[3], uvs[2], uvs[1], uvs[0]];
        }

        let mut vertices = Vec::new();

        let texture = self
            .textures
            .get(sprite.texture_id)
            .expect("Missing texture id");
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

        self.meshes2d.push((mesh, transform.translation));
    }

    pub fn draw_debug_rect(&mut self, rect: &Rect, color: Color) {
        self.debug_meshes.push(DebugMesh::rect(rect, color));
    }

    pub fn draw_debug_line(&mut self, p1: Vec2, p2: Vec2, color: Color) {
        self.debug_meshes.push(DebugMesh::line(p1, p2, color));
    }
}
