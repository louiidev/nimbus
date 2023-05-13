use glam::{Vec2, Vec3};
use rand::thread_rng;
use std::ops::Range;

use super::sprite::Sprite;
use crate::{arena::ArenaId, renderer::Renderer, Engine};
use rand::prelude::*;

const DEFAULT_MIN_PARTICLE_LIFETIME: f32 = 2f32;
const DEFAULT_MAX_PARTICLE_LIFETIME: f32 = 5f32;
const DEFAULT_EMISSION_RATE: usize = 5;

pub struct Particle {
    position: Vec3,
    velocity: Vec3,
    time_left: f32,
    duration: f32,
    alpha: f32,
}

#[derive(Default)]
pub struct ParticleEmitter {
    texture_id: ArenaId,
    particles: Vec<Particle>,
    position: Vec3,
    duration: Option<f32>,
    rng: ThreadRng,
    particle_lifetime: Range<f32>,
    emission_rate: usize,
}

pub struct ParticleBuilder {
    texture_id: ArenaId,
    position: Vec3,
    size: Vec2,
    lifetime: Range<f32>,
    emission_rate: usize,
    duration: Option<f32>,
}

impl ParticleBuilder {
    pub fn new(position: Vec3) -> ParticleBuilder {
        ParticleBuilder {
            texture_id: ArenaId::first(),
            position,
            size: Vec2::splat(1.),
            lifetime: DEFAULT_MIN_PARTICLE_LIFETIME..DEFAULT_MAX_PARTICLE_LIFETIME,
            emission_rate: DEFAULT_EMISSION_RATE,
            duration: None,
        }
    }

    pub fn set_duration(mut self, duration: f32) -> ParticleBuilder {
        self.duration = Some(duration);
        self
    }

    pub fn set_emission_rate(mut self, emission_rate: usize) -> ParticleBuilder {
        self.emission_rate = emission_rate;
        self
    }

    pub fn set_lifetime(mut self, lifetime: Range<f32>) -> ParticleBuilder {
        self.lifetime = lifetime;
        self
    }

    pub fn set_size(mut self, size: Vec2) -> ParticleBuilder {
        self.size = size;
        self
    }

    pub fn set_texture(mut self, texture: ArenaId) -> ParticleBuilder {
        self.texture_id = texture;
        self
    }

    pub fn build(self) -> ParticleEmitter {
        ParticleEmitter {
            texture_id: self.texture_id,
            particles: Vec::default(),
            position: self.position,
            duration: self.duration,
            rng: thread_rng(),
            particle_lifetime: self.lifetime,
            emission_rate: self.emission_rate,
        }
    }
}

impl ParticleEmitter {
    pub fn spawn(&mut self, velocity: Vec3, time_left: f32) {
        self.particles.push(Particle {
            position: self.position,
            velocity,
            time_left,
            duration: time_left,
            alpha: 255.,
        })
    }

    pub fn update(&mut self, engine: &mut Engine, delta: f32) {
        for _ in 0..self.emission_rate {
            let time_left = self.rng.gen_range(self.particle_lifetime.clone());
            let velocity = Vec3 {
                x: self.rng.gen_range(-30..30) as f32,
                y: self.rng.gen_range(-70..-20) as f32,
                z: 0.,
            };
            self.spawn(velocity, time_left)
        }

        for particle in &mut self.particles {
            particle.position += particle.velocity * delta;
            particle.time_left -= delta;
            let progress = (particle.duration - particle.time_left) / particle.duration;
            particle.alpha = (1.0 - progress) * 255.;
        }

        self.particles.retain(|p| p.time_left > 0.);
    }

    pub fn render(&mut self, renderer: &mut Renderer, delta: f32) {
        for particle in &mut self.particles {
            let mut sprite: Sprite = self.texture_id.into();
            sprite.color.set_a(particle.alpha / 255.);
            renderer.draw_sprite_basic(&sprite, particle.position);
        }
    }
}
