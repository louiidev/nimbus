use glam::Vec2;

use nimbus::{Engine, Nimbus, Sprite, Transform};

fn main() {
    run();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        }
    }

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
        let handle = engine.load_texture_bytes(include_bytes!("../assets/cloud.png"), "png");

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

        // engine.ui.text(
        //     Text {
        //         value: "Nimbus web build".to_string(),
        //         ..Default::default()
        //     },
        //     Vec2::default(),
        // );

        use nimbus::input::Axis::*;
        move_direction += Vec2::new(engine.get_axis(LeftX), engine.get_axis(LeftY));

        if move_direction != Vec2::default() {
            self.player.1.position += move_direction.normalize().extend(0.) * delta * 150f32;
        }
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, delta: f32) {
        renderer.draw_sprite(self.player.0, self.player.1)
    }
}
