use glam::Vec3;
use render_buddy::sprite::Sprite;

use crate::renderer::Renderer;

pub struct Particle {
    position: Vec3,
    velocity: Vec3,
    time_left: f32,
}

pub struct ParticleEmitter {
    sprite: Sprite,
    particles: Vec<Particle>,
    position: Vec3,
}

impl ParticleEmitter {
    pub fn new(sprite: Sprite, position: Vec3) -> Self {
        Self {
            sprite,
            particles: Vec::default(),
            position,
        }
    }

    pub fn spawn(&mut self, velocity: Vec3, life_length: f32) {
        self.particles.push(Particle {
            position: self.position,
            velocity,
            time_left: life_length,
        })
    }

    pub fn render(&mut self, renderer: &mut Renderer, delta: f32) {
        let mut dead_particles = Vec::default();

        for (index, particle) in &mut self.particles.iter_mut().enumerate() {
            particle.position += particle.velocity * delta;
            particle.time_left -= delta;
            renderer.draw_sprite_basic(&self.sprite, particle.position);

            if particle.time_left <= 0. {
                dead_particles.push(index);
            }
        }

        // for dead in dead_particles [
        //     self.particles.remove(dead);
        // ]
    }
}
