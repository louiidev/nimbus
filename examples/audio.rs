use nimbus::{arena::ArenaId, Engine, Nimbus};

#[derive(Default)]
pub struct Game {
    audio_id: ArenaId,
}

impl Nimbus for Game {
    fn init(&mut self, engine: &mut Engine) {
        let id = engine.load_audio("./examples/Windless Slopes.ogg");
        engine.audio.play(id);

        self.audio_id = id;
    }

    fn update(&mut self, engine: &mut Engine, _delta: f32) {
        use sdl2::keyboard::Keycode::*;
        if engine.input.keyboards_inputs.just_pressed(Space) {
            if engine.audio.paused(self.audio_id) {
                engine.audio.play(self.audio_id);
            } else {
                engine.audio.pause(self.audio_id);
            }
        }
    }

    fn render(&mut self, _renderer: &mut nimbus::renderer::Renderer, _delta: f32) {}
}

fn main() {
    let app = Engine::default();
    let game = Game::default();
    app.run(game);
}
