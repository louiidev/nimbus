use glam::{Mat4, Vec2, Vec3, Vec4Swizzles};

use crate::components::{color::Color, text::Text, transform::Transform};

use self::layout::{Layout, LayoutDirection, LayoutTheme};

use super::{
    font_renderer::{self, FontRenderer},
    mesh2d::{self, Mesh2d, Vertex2D, QUAD_INDICES, QUAD_VERTEX_POSITIONS},
    Renderer,
};

pub mod layout;

pub struct Ui {
    pub(crate) renderer: Option<Renderer>,
    pub(crate) font_renderer: FontRenderer,
    current_layout: Vec<Layout>,
    pub(crate) render_meta: Vec<Mesh2d>,
}

impl Ui {
    pub fn new(canvas_size: Vec2) -> Self {
        Self {
            current_layout: vec![Layout {
                size: canvas_size,
                layout_direction: LayoutDirection::Horizontal,
                ..Default::default()
            }],
            font_renderer: FontRenderer::new(),
            render_meta: Vec::default(),
            renderer: None,
        }
    }
    pub fn get_available_space(&self) -> Vec2 {
        let layout = self.current_layout.last().expect("Missing layout");
        match layout.layout_direction {
            LayoutDirection::Horizontal => {
                Vec2::new(layout.width() - layout.allocated_space, layout.height())
            }
            LayoutDirection::Vertical => {
                Vec2::new(layout.width(), layout.height() - layout.allocated_space)
            }
        }
    }

    pub fn get_next_available_position(&self) -> Vec2 {
        let layout = self.current_layout.last().expect("Missing layout");
        match layout.layout_direction {
            LayoutDirection::Horizontal => Vec2::new(
                layout.position.x + layout.allocated_space,
                layout.position.y,
            ),
            LayoutDirection::Vertical => Vec2::new(
                layout.position.x,
                layout.position.y + layout.allocated_space,
            ),
        }
    }

    pub fn allocate_space(&mut self, size: Vec2) {
        let layout = self.current_layout.last_mut().unwrap();

        layout.allocated_space += match layout.layout_direction {
            LayoutDirection::Horizontal => size.x,
            LayoutDirection::Vertical => size.y,
        }
    }

    pub fn left_panel(&mut self, width: f32, callback: impl FnOnce(&mut Self)) {
        let available_space = self.get_available_space();
        let size = Vec2::new(width, available_space.y);
        let layout = Layout {
            size,
            layout_theme: Some(LayoutTheme {
                color: Color::BLACK,
                padding: 0.,
            }),
            ..Default::default()
        };

        self.push_layout(layout);
        callback(self);
        self.pop_layout();
    }

    pub fn right_panel(&mut self, width: f32, callback: impl FnOnce(&mut Self)) {
        let available_space = self.get_available_space();
        let size = Vec2::new(width, available_space.y);
        let layout = Layout {
            size,
            layout_theme: Some(LayoutTheme::default()),
            ..Default::default()
        };

        self.push_layout(layout);
        callback(self);
        self.pop_layout();
    }

    pub fn panel(&mut self, callback: impl FnOnce(&mut Self)) {
        let available_space = self.get_available_space();
        let layout = Layout {
            size: available_space,
            layout_theme: Some(LayoutTheme::default()),
            position: self.get_next_available_position(),
            ..Default::default()
        };

        self.push_layout(layout);
        callback(self);
        self.pop_layout();
    }

    pub fn label(&mut self, text: &str) {
        let size = self.text(
            Text {
                value: text.into(),
                ..Default::default()
            },
            self.get_next_available_position(),
        );

        self.allocate_space(size);

        // let mesh = Mesh2d::rect(position, size, color);
        // self.text(Text {
        //     value: text.to_string(),
        //     ..Default::default()
        // });
    }

    pub fn text(&mut self, text: Text, position: Vec2) -> Vec2 {
        let container_size = self.get_available_space();
        dbg!(&container_size);
        let renderer = self.renderer.as_mut().unwrap();
        let text_glyphs = self.font_renderer.queue_text(
            &text,
            container_size,
            fontdue::layout::CoordinateSystem::PositiveYDown,
            &mut renderer.textures,
            &renderer.device,
            &renderer.queue,
        );

        let mut size = Vec2::default();

        let transform = Transform::from_xyz(position.x, position.y, 0.0);

        for text_glyph in text_glyphs {
            let current_image_size = text_glyph.atlas_info.atlas_size;
            let scale_factor = 1f32;

            let extracted_transform = transform.compute_matrix()
                * Mat4::from_scale(Vec3::splat(scale_factor.recip()))
                * Mat4::from_translation(text_glyph.position.extend(0.));

            let rect = text_glyph.rect;

            let mut vertices = Vec::new();

            let uvs = [
                Vec2::new(rect.min.x, rect.min.y),
                Vec2::new(rect.max.x, rect.min.y),
                Vec2::new(rect.max.x, rect.max.y),
                Vec2::new(rect.min.x, rect.max.y),
            ]
            .map(|pos| pos / current_image_size);

            let positions = QUAD_VERTEX_POSITIONS.map(|pos| {
                (extracted_transform
                    * ((pos - Vec2::new(-0.5, -0.5)) * rect.size())
                        .extend(0.)
                        .extend(1.))
                .xyx()
                .into()
            });
            for i in 0..QUAD_VERTEX_POSITIONS.len() {
                vertices.push(Vertex2D {
                    position: positions[i],
                    uv: uvs[i].into(),
                    color: text.theme.color.as_rgba_f32(),
                });
            }

            let glyph_position = text_glyph.position - -Vec2::new(-0.5, -0.5);

            let x_distance = glyph_position.x - size.x;
            let actual_glyph_size = rect.size();

            size.y = size.y.max(actual_glyph_size.y);
            size.x += actual_glyph_size.x + x_distance;
            let meta = Mesh2d {
                texture_id: text_glyph.atlas_info.texture_id,
                vertices,
                indices: QUAD_INDICES.to_vec(),
            };
            let layout = self.current_layout.last_mut().unwrap();

            layout.children.push(meta);
        }

        size
    }

    fn push_widget(&mut self, size: Vec2, mesh: Mesh2d) {}

    fn push_layout(&mut self, layout: Layout) {
        self.allocate_space(layout.size);
        self.current_layout.push(layout);
    }

    fn pop_layout(&mut self) {
        let mut layout = self
            .current_layout
            .pop()
            .expect("Missing layout when popping");

        self.render_meta.append(&mut layout.get_render_meta());
    }

    pub(crate) fn resize(&mut self, canvas_size: Vec2) {
        self.current_layout[0].size = canvas_size;
    }

    pub(crate) fn reset(&mut self) {
        self.current_layout.drain(1..); // Drop all back first item
        self.current_layout[0].allocated_space = 0.;
        self.render_meta = Vec::default();
    }
}
