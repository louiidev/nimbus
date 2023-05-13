use glam::Vec3;
use nimbus::{
    arena::ArenaId,
    components::particles::{ParticleBuilder, ParticleEmitter},
    renderer::Renderer,
    Engine, Nimbus,
};

pub struct State {
    particle_emitter: ParticleEmitter,
}

impl Nimbus for State {
    fn update(&mut self, engine: &mut Engine, delta: f32) {
        self.particle_emitter.update(engine, delta);
    }

    fn render(&mut self, renderer: &mut Renderer, delta: f32) {
        self.particle_emitter.render(renderer, delta);
    }
}

fn main() {
    let mut app = Engine::default();

    let particle_texture = app.load_texture("particle.aseprite");

    let particle_emitter = ParticleBuilder::new(Vec3::ZERO)
        .set_texture(particle_texture)
        .set_emission_rate(1)
        .build();

    let state = State { particle_emitter };
    app.run(state);
}
