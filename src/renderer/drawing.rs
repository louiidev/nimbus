use std::collections::BTreeMap;

use glam::{Vec2, Vec3};

use crate::{
    arena::ArenaId,
    components::{color::Color, line::Line2D},
    cube::Cube,
    fonts::PositionedGlyph,
    line::LineMeshBuilder,
    mesh::{
        AttributeValue, Indices, Mesh, MeshAttribute, MeshBuilder, Vertex, QUAD_INDICES, QUAD_UVS,
        QUAD_VERTEX_POSITIONS,
    },
    model::Model,
    sprite::Anchor,
};

use super::{rect::Rect, sprite::Sprite, text::Text, transform::Transform, Renderer};

impl Renderer {
    pub fn set_sorting_axis_2d(&mut self, sorting_axis: Vec3) {
        self.sorting_axis = sorting_axis;
    }

    pub fn rect_to_mesh(&mut self, rect: &Rect, color: Color) -> Mesh {
        let quad_size = rect.size();
        let transform = Transform::from_position(rect.min.extend(0.));
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

        let material_handle = self.material_map.default;

        Mesh::new(
            Some(ArenaId::first()),
            material_handle,
            vertices,
            Indices::U16(QUAD_INDICES.to_vec()),
            (transform.position * self.sorting_axis).length(),
        )
    }

    pub fn draw_rect(&mut self, rect: &Rect, color: Color) {
        let mesh = self.rect_to_mesh(rect, color);

        self.push(mesh);
    }

    pub fn draw_sprite_basic(&mut self, sprite: &Sprite, position: Vec3) {
        self.draw_sprite(sprite, Transform::from_position(position));
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite, mut transform: Transform) {
        let texture = self
            .textures
            .get(sprite.handle)
            .expect("Mesh is missing texture");

        let mut uvs = QUAD_UVS;

        if sprite.flip_x {
            uvs = [uvs[1], uvs[0], uvs[3], uvs[2]];
        }
        if sprite.flip_y {
            uvs = [uvs[3], uvs[2], uvs[1], uvs[0]];
        }

        let current_image_size = texture.dimensions;

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

        let sort_value = transform.position * self.sorting_axis;

        transform.position.z = 0.;

        let positions: [[f32; 3]; 4] = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
            transform
                .transform_point(((quad_pos - sprite.anchor.as_vec()) * quad_size).extend(0.))
                .into()
        });

        let vertices = positions
            .iter()
            .zip(uvs)
            .map(|(position, uv)| {
                Vertex(BTreeMap::from([
                    (MeshAttribute::Position, AttributeValue::Position(*position)),
                    (MeshAttribute::UV, AttributeValue::UV(uv.into())),
                    (MeshAttribute::Color, AttributeValue::Color(sprite.color)),
                ]))
            })
            .collect();

        let material_handle = sprite.material.unwrap_or(self.material_map.default);

        let mut mesh = MeshBuilder::new()
            .with_indices(crate::mesh::Indices::U16(QUAD_INDICES.to_vec()))
            .with_vertices(vertices)
            .with_material(material_handle)
            .with_texture(sprite.handle)
            .build();

        mesh.sort_value = sort_value.x + sort_value.y + sort_value.z;

        self.push(mesh);
    }

    pub fn draw_text(&mut self, text: &Text, mut transform: Transform) {
        let positioned_glyphs = self.get_positioned_glyphs(text, None);
        let size = positioned_glyphs.iter().fold(
            Vec2::default(),
            |mut size: Vec2, text_glyph: &PositionedGlyph| {
                let rect = text_glyph.rect;
                let glyph_position = text_glyph.position;

                let x_distance = glyph_position.x - size.x;
                let actual_glyph_size = rect.size();
                size.y = size.y.max(actual_glyph_size.y);
                size.x += actual_glyph_size.x + x_distance;

                size
            },
        );

        let offset = Vec2::new(size.x / 2. * transform.scale.x, -size.y * transform.scale.y);
        transform.position -= offset.extend(0.);

        let meshes: Vec<super::mesh::Mesh> = positioned_glyphs
            .iter()
            .map(|text_glyph| {
                // let transform = Transform::from_translation(position + text_glyph.position.extend(0.));
                let mut transform = transform.clone();
                transform.position = transform.transform_point(text_glyph.position.extend(0.));
                let current_image_size = text_glyph.atlas_info.atlas_size;
                // let scale_factor = 1f32;
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
                        .transform_point(
                            ((quad_pos - Vec2::new(-0.5, -0.5)) * rect.size()).extend(0.),
                        )
                        .into()
                });

                for i in 0..QUAD_VERTEX_POSITIONS.len() {
                    vertices.push(Vertex(BTreeMap::from([
                        (
                            MeshAttribute::Position,
                            AttributeValue::Position(positions[i]),
                        ),
                        (MeshAttribute::UV, AttributeValue::UV(uvs[i].into())),
                        (
                            MeshAttribute::Color,
                            AttributeValue::Color(text.color.into()),
                        ),
                    ])));
                }

                let material_handle = text.material.unwrap_or(self.material_map.default);

                Mesh {
                    texture_handle: Some(text_glyph.atlas_info.texture_handle),
                    material_handle,
                    vertices,
                    indices: crate::mesh::Indices::U16(QUAD_INDICES.to_vec()),
                    sort_value: transform.position.z,
                    batch: true,
                }
            })
            .collect();

        self.append(meshes);
    }

    pub fn draw_text_basic(&mut self, text: &Text, position: Vec3) {
        self.draw_text(text, Transform::from_position(position));
    }

    pub fn draw_line(&mut self, line: &Line2D, color: &Color) {
        self.push(LineMeshBuilder::new().line(line.0.extend(0.), line.1.extend(0.), color));
    }

    pub fn draw_line_rect(&mut self, rect: &Rect, color: &Color) {
        self.push(LineMeshBuilder::new().rect(rect, color.into()));
    }

    pub fn draw_model(&mut self, model: &Model, transform: Transform) {
        let mut mesh_builder = MeshBuilder::new()
            .with_texture(model.texture)
            .with_indices(Indices::U32(model.indices.clone()))
            .with_material(model.material)
            .with_batch(false)
            .with_attributes(
                MeshAttribute::Position,
                model
                    .positions
                    .iter()
                    .map(|v| AttributeValue::Position(transform.transform_point(*v).into()))
                    .collect(),
            );

        if !model.colors.is_empty() {
            mesh_builder = mesh_builder.with_attributes(
                MeshAttribute::Color,
                model
                    .colors
                    .iter()
                    .map(|v| AttributeValue::Color((*v).into()))
                    .collect(),
            );
        } else {
            mesh_builder = mesh_builder.with_attribute(
                MeshAttribute::Color,
                AttributeValue::Color(Color::WHITE.as_rgba_f32()),
            );
        }

        if !model.tex_coords.is_empty() {
            mesh_builder = mesh_builder.with_attributes(
                MeshAttribute::UV,
                model
                    .tex_coords
                    .iter()
                    .map(|v| AttributeValue::UV((*v).into()))
                    .collect(),
            );
        }

        self.push(mesh_builder.build());
    }

    // pub fn draw_cube(&mut self, cube: &Cube, transform: Transform) {
    //     let mesh = MeshBuilder::new()
    //         .with_attributes(
    //             MeshAttribute::Position,
    //             CUBE_VERTEX_POSITIONS
    //                 .iter()
    //                 .map(|v| {
    //                     AttributeValue::Position(transform.transform_point(*v * cube.size).into())
    //                 })
    //                 .collect(),
    //         )
    //         .with_attributes(
    //             MeshAttribute::UV,
    //             CUBE_VERTEX_UVS
    //                 .iter()
    //                 .map(|uv| AttributeValue::UV(uv.to_array()))
    //                 .collect(),
    //         )
    //         .with_attribute(
    //             MeshAttribute::Color,
    //             AttributeValue::Color(cube.color.into()),
    //         )
    //         .with_material(cube.material)
    //         .with_texture(cube.texture)
    //         .build();

    //     self.meshes.push(mesh);
    // }

    // pub fn measure_text(&mut self, text: &Text) -> Vec2 {

    // }
}
