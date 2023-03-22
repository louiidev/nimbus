use std::{collections::HashMap, hash::Hash};

use crate::time::Time;

use super::{sprite::Sprite, texture_atlas::TextureAtlas, timer::Timer};

#[derive(Default, Debug)]
pub struct AnimatedSprite<S> {
    pub sprite: Sprite,
    pub atlas: TextureAtlas,
    pub timer: Timer,
    pub current_state: S,
    pub current_frame: usize,
    pub states: HashMap<S, AnimatedState<S>>,
}

impl<S: PartialEq + Eq + Hash + Clone + Copy> AnimatedSprite<S> {
    pub fn new(
        sprite: Sprite,
        atlas: TextureAtlas,
        states: HashMap<S, AnimatedState<S>>,
        current_state: S,
        current_frame: usize,
    ) -> Self {
        Self {
            sprite,
            atlas,
            timer: Timer::default(),
            current_state,
            current_frame,
            states,
        }
    }

    pub fn set_animation_state(&mut self, new_state: S) {
        if new_state != self.current_state {
            let state = self.states.get(&new_state).unwrap();
            self.current_state = new_state;
            self.current_frame = state.animation_frames_indices[0];
        }
    }

    pub fn animate(&mut self, time: &Time) {
        if let Some(animation_state) = self.states.get(&self.current_state).cloned() {
            self.timer.tick(time.delta());
            if self.timer.just_finished() {
                let last_frame_index = animation_state.animation_frames_indices.len() - 1;
                if self.current_frame == last_frame_index {
                    if let Some(next_animation_state) = animation_state.on_end_animation_state {
                        self.set_animation_state(next_animation_state);
                    } else {
                        self.current_frame = animation_state.animation_frames_indices[0];
                    }
                } else {
                    self.current_frame += 1;
                }

                let sprite_rect = self.atlas.textures.get(self.current_frame).copied();
                self.sprite.texture_rect = sprite_rect;
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
