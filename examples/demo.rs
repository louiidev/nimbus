use glam::Vec2;
use nimbus::{Engine, Nimbus, Transform};
use render_buddy::sprite::Sprite;

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
        let mut move_direction = Vec2::default();
        for key in engine.get_pressed() {
            use nimbus::input::Input::*;
            match key {
                W => move_direction += Vec2::Y,
                S => move_direction -= Vec2::Y,
                A => move_direction -= Vec2::X,
                D => move_direction += Vec2::X,
                _ => {}
            }
        }

        use nimbus::input::Axis::*;
        move_direction += Vec2::new(engine.get_axis(LeftX), engine.get_axis(LeftY));

        if move_direction != Vec2::default() {
            self.player.1.position += move_direction.normalize().extend(0.) * delta * 150f32;
        }
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, _delta: f32) {
        renderer.draw_sprite(self.player.0, self.player.1)
    }
}
