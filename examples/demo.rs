use glam::Vec2;
use nimbus::{
    components::{sprite::Sprite, transform::Transform},
    Engine, Nimbus,
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

impl Nimbus for GameExample {
    fn init(&mut self, engine: &mut Engine) {
        let texture_id = engine.load_texture("./examples/cloud.aseprite");

        self.player.0.texture_id = texture_id;
    }

    fn update(&mut self, engine: &mut Engine, delta: f32) {
        let mut move_direction = Vec2::default();
        for key in engine.input.keyboards_inputs.get_pressed() {
            use sdl2::keyboard::Keycode::*;
            match key {
                W => move_direction += Vec2::Y,
                S => move_direction -= Vec2::Y,
                A => move_direction -= Vec2::X,
                D => move_direction += Vec2::X,
                _ => {}
            }
        }

        if move_direction != Vec2::default() {
            self.player.1.translation += move_direction.normalize().extend(0.) * delta * 150f32;
        }

        // egui::SidePanel::right("egui_demo_panel")
        //     .resizable(false)
        //     .default_width(150.)
        //     .show(&engine.egui_ctx(), |ui| {
        //         ui.vertical_centered(|ui| {
        //             ui.heading("✒ egui demos");
        //         });
        //     });
    }

    fn render(&mut self, renderer: &mut nimbus::renderer::Renderer, delta: f32) {
        renderer.draw_sprite(&self.player.0, &self.player.1)
    }
}
