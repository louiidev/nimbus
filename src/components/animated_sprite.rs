use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::time::Time;

use super::{
    rect::Rect,
    sprite::Sprite,
    texture_atlas::TextureAtlas,
    timer::{self, Timer},
};

#[derive(Default, Debug)]
pub struct AnimatedSprite<S> {
    pub sprite: Sprite,
    pub atlas: TextureAtlas,
    pub timer: Timer,
    pub current_state: S,
    pub current_frame_index: usize,
    pub states: HashMap<S, AnimatedState<S>>,
}

impl<S: PartialEq + Eq + Hash + Clone + Copy + Debug> AnimatedSprite<S> {
    pub fn new(
        sprite: Sprite,
        atlas: TextureAtlas,
        states: HashMap<S, AnimatedState<S>>,
        current_state: S,
    ) -> Self {
        Self {
            sprite,
            atlas,
            timer: Timer::default(),
            current_state,
            current_frame_index: 0,
            states,
        }
    }

    pub fn init(&mut self, seconds_per_frame: f32, states: HashMap<S, AnimatedState<S>>) {
        self.set_sprite_texture_rect(states[&self.current_state].clone());

        self.states = states;

        self.timer = Timer::from_seconds(seconds_per_frame, timer::TimerMode::Repeating);
    }

    pub fn set_animation_state(&mut self, new_state: S) {
        if new_state != self.current_state {
            self.current_state = new_state;
            self.current_frame_index = 0;
        }
    }

    pub fn set_sprite_texture_rect(&mut self, animation_state: AnimatedState<S>) {
        let frame = animation_state.animation_frames_indices[self.current_frame_index];
        let sprite_rect = self.atlas.textures.get(frame).copied();
        self.sprite.texture_rect = sprite_rect;
    }

    pub fn animate(&mut self, time: &Time) {
        if let Some(mut animation_state) = self.states.get(&self.current_state).cloned() {
            self.timer.tick(time.delta());
            if self.timer.just_finished() {
                let last_frame_index = animation_state.animation_frames_indices
                    [animation_state.animation_frames_indices.len() - 1];
                if self.current_frame_index == last_frame_index {
                    if let Some(next_animation_state) = animation_state.on_end_animation_state {
                        self.set_animation_state(next_animation_state);
                        animation_state = self.states.get(&next_animation_state).unwrap().clone();
                    } else {
                        self.current_frame_index = 0;
                    }
                } else {
                    self.current_frame_index += 1;
                }

                self.set_sprite_texture_rect(animation_state);
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct AnimatedState<S> {
    pub on_end_animation_state: Option<S>,
    pub animation_frames_indices: Vec<usize>,
}

impl<S> AnimatedState<S> {
    pub fn new(animation_frames_indices: Vec<usize>) -> Self {
        Self {
            on_end_animation_state: None,
            animation_frames_indices,
        }
    }
}
