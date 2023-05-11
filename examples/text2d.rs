use glam::{Vec2, Vec3};
use nimbus::{
    arena::ArenaId,
    components::{color::Color, rect::Rect, text::TextTheme, transform::Transform},
    Engine, Nimbus,
};

#[derive(Default)]
pub struct Game {
    texture_id: ArenaId,
    text_transform: Transform,
}

impl Nimbus for Game {
    fn init(&mut self, engine: &mut Engine) {
        self.texture_id = engine.load_texture_bytes(include_bytes!("../assets/cloud.png"), "png");
    }

    fn update(&mut self, engine: &mut Engine, _delta: f32) {
        self.text_transform.scale = Vec3::splat((engine.time.elapsed_seconds().sin() + 1.1) * 2.0);
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, _delta: f32) {
        renderer.draw_text(
            &nimbus::components::text::Text {
                alignment: nimbus::components::text::TextAlignment {
                    vertical: fontdue::layout::VerticalAlign::Bottom,
                    horizontal: fontdue::layout::HorizontalAlign::Right,
                },
                value: "Testing World Hello".to_string(),
                theme: TextTheme {
                    font_size: 40.,
                    color: Color::GREEN,
                },
                ..Default::default()
            },
            self.text_transform,
        );
        renderer.draw_rect(
            &Rect::from_center_size(Vec2::ZERO, Vec2::splat(8.)),
            Color::RED,
        );

        renderer.draw_text_basic("Hello Hello", Vec2::new(150., 150.));
        renderer.draw_rect(
            &Rect::from_center_size(Vec2::new(150., 150.), Vec2::splat(8.)),
            Color::GREEN,
        );
    }
}

fn main() {
    let app = Engine::default();
    let game = Game::default();
    app.run(game);
}
