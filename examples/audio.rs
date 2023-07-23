use nimbus::{arena::ArenaId, audio::AudioSource, renderer::Renderer, Engine, Nimbus};

#[derive(Default)]
pub struct Game {
    audio: ArenaId<AudioSource>,
}

impl Nimbus for Game {
    fn init(&mut self, engine: &mut Engine) {
        let handle = engine.load_audio("Windless Slopes.ogg");
        engine.audio.play(handle);

        self.audio = handle;
    }

    fn update(&mut self, engine: &mut Engine, _delta: f32) {
        use nimbus::input::Input::*;
        if engine.just_pressed(Space) {
            if engine.audio.paused(self.audio) {
                engine.audio.play(self.audio);
            } else {
                engine.audio.pause(self.audio);
            }
        }
    }

    fn render(&mut self, _renderer: &mut Renderer, _delta: f32) {}
}

fn main() {
    let app = Engine::default();
    let game = Game::default();
    app.run(game);
}
