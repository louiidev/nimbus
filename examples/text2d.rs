use glam::Vec3;
use nimbus::{components::color::Color, Engine, Nimbus, Transform};
use render_buddy::text::Text;

#[derive(Default)]
pub struct Game {
    text_transform: Transform,
}

impl Nimbus for Game {
    fn update(&mut self, engine: &mut Engine, _delta: f32) {
        self.text_transform.scale = Vec3::splat((engine.time.elapsed_seconds().sin() + 1.1) * 2.0);
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, _delta: f32) {
        renderer.draw_text(
            Text::new("Hello world", 40.).with_color(Color::GREEN.into()),
            self.text_transform,
        );

        renderer.draw_text_basic(Text::new("Testing", 32.), Vec3::new(150., 150., 0.));
    }
}

fn main() {
    let app = Engine::default();
    let game = Game::default();
    app.run(game);
}
