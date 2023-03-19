use glam::Vec2;
use guacamole::{
    components::{sprite::Sprite, transform::Transform},
    Engine, Guacamole,
};

fn main() {
    let app = Engine::default();
    let game = GameExample::default();
    app.run(game);
}

#[derive(Debug, Default)]
pub struct GameExample {
    player: (Sprite, Transform),
}

impl Guacamole for GameExample {
    fn init(&mut self, engine: &mut Engine) {
        let texture_id = engine
            .renderer
            .load_texture(include_bytes!("../examples/fullTiles.png"));

        self.player.0.texture_id = texture_id;
    }

    fn update(&mut self, engine: &mut Engine) {
        let mut move_direction = Vec2::default();
        for key in engine.input.keyboards_inputs.get_pressed() {
            use winit::event::VirtualKeyCode::*;
            match key {
                W => move_direction += Vec2::Y,
                S => move_direction -= Vec2::Y,
                A => move_direction -= Vec2::X,
                D => move_direction += Vec2::X,
                _ => {}
            }
        }
        if move_direction != Vec2::default() {
            self.player.1.translation +=
                move_direction.normalize().extend(0.) * engine.time.delta_seconds() * 150f32;
        }

        engine.renderer.draw_sprite(&self.player.0, &self.player.1)
    }
}
