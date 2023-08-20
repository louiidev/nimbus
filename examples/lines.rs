use std::mem;

use glam::{Vec2, Vec3};
use nimbus::{
    components::{
        color::Color,
        line::{Line, Line2D},
    },
    rect::Rect,
    sprite::Sprite,
    utils::collisions::{line_line_collision, line_rectangle_collision},
    Engine, Nimbus,
};

fn main() {
    let app = Engine::default();
    let game = GameExample::default();
    app.run(game);
}

#[derive(Debug, Default)]
pub struct GameExample {
    rect: Rect,
    mouse_line: Line,
}

impl Nimbus for GameExample {
    fn init(&mut self, engine: &mut Engine) {
        let handle = engine.load_texture("cloud.png");

        self.rect = Rect::from_center_size(Vec2::new(0., 150.), Vec2::splat(50.));

        self.mouse_line = Line(Vec3::ZERO, Vec3::new(50., 50., 0.));
    }

    fn update(&mut self, engine: &mut Engine, delta: f32) {
        if let Some(pos) = engine
            .camera
            .viewport_to_world_position(engine.input.mouse_position, engine.get_viewport())
        {
            self.mouse_line.1 = pos.origin;
        }
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, _delta: f32) {
        // renderer.draw_line(&self.mouse_line, &Color::GREEN);

        // if let Some(collision) = line_rectangle_collision(self.mouse_line, self.rect) {
        //     renderer.draw_line(&self.mouse_line, &Color::RED);
        //     renderer.draw_line_rect(
        //         &Rect::from_center_size(collision, Vec2::splat(5.)),
        //         &Color::RED,
        //     )
        // } else {
        //     renderer.draw_line(&self.mouse_line, &Color::GREEN);
        // }
        // renderer.draw_line_rect(&self.rect, &Color::BLUE);

        let rect = Rect::from_corners(Vec2::ZERO, Vec2::X + Vec2::Y);

        let color = if point_in_quad(rect, self.mouse_line.1) {
            Color::RED
        } else {
            Color::BLUE
        };

        renderer.draw_line_from_points(
            &vec![Vec3::ZERO, Vec3::Y, Vec3::X + Vec3::Y, Vec3::X, Vec3::ZERO],
            &color,
        );
    }
}

pub fn point_in_triangle(mut a: Vec3, mut b: Vec3, c: Vec3, point: Vec3) -> bool {
    if a.y == c.y {
        mem::swap(&mut a, &mut b)
    }

    let s1 = c.y - a.y;
    let s2 = c.x - a.x;
    let s3 = b.y - a.y;
    let s4 = point.y - a.y;

    let w1 = (a.x * s1 + s4 * s2 - point.x * s1) / (s3 * s2 - (b.x - a.x) * s1);
    let w2 = (s4 - w1 * s3) / s1;

    if w1.is_nan() || w2.is_nan() {
        // panic so we can detect bug, otherwise might be hard to catch whats happening
        panic!("Error in maths for p in triangle");
    }

    w1 >= 0. && w2 >= 0. && (w1 + w2) <= 1.
}

struct Quad {}

pub fn point_in_quad(rect: Rect, point: Vec3) -> bool {
    // [0, 2, 3, 0, 1, 2]
    let tri_a = vec![
        rect.min.extend(0.),
        rect.max.extend(0.),
        Vec3::new(rect.min.x, rect.max.y, 0.),
    ];

    let tri_b = vec![
        rect.min.extend(0.),
        Vec3::new(rect.max.x, rect.min.y, 0.),
        rect.max.extend(0.),
    ];

    return point_in_triangle(tri_a[0], tri_a[1], tri_a[2], point)
        || point_in_triangle(tri_b[0], tri_b[1], tri_b[2], point);
}

// public static bool PointInTriangle(Point A, Point B, Point C, Point P)
// 	{
// 			double s1 = C.y - A.y;
// 			double s2 = C.x - A.x;
// 			double s3 = B.y - A.y;
// 			double s4 = P.y - A.y;

// 			double w1 = (A.x * s1 + s4 * s2 - P.x * s1) / (s3 * s2 - (B.x-A.x) * s1);
// 			double w2 = (s4- w1 * s3) / s1;
// 			return w1 >= 0 && w2 >= 0 && (w1 + w2) <= 1;
// 	}
