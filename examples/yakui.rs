use glam::Vec2;
use nimbus::{sprite::Sprite, transform::Transform, yakui, Engine, Nimbus};

fn main() {
    let app = Engine::default();
    let game = GameExample::default();
    app.run(game);
}

#[derive(Debug, Default)]
pub struct GameExample {
    player: (Sprite, Transform),
}

impl Nimbus for GameExample {
    fn init(&mut self, engine: &mut Engine) {
        let handle = engine.load_texture("cloud.png");

        self.player.0 = Sprite::new(handle);
    }

    fn update(&mut self, engine: &mut Engine, delta: f32) {
        yakui::column(|| {
            yakui::text(32.0, "Hello, world!");

            if yakui::button("Click me!").clicked {
                println!("Button clicked.");
            }
        });
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, _delta: f32) {
        renderer.draw_sprite(&self.player.0, self.player.1)
    }
}
