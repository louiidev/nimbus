use super::timer::{self, Timer};
use crate::arena::ArenaId;
use crate::renderer::{sprite::Sprite, texture::Texture, texture_atlas::TextureAtlas};
use crate::{asset_loader::FrameData, time::Time};
use glam::Vec2;
use std::{collections::HashMap, fmt::Debug, hash::Hash, str::FromStr};

#[derive(Default, Debug, Clone)]
pub struct AnimatedSprite<S> {
    pub sprite: Sprite,
    pub atlas: TextureAtlas,
    pub timer: Timer,
    pub current_state: S,
    pub current_frame_index: usize,
    pub states: HashMap<S, AnimatedState<S>>,
}

impl<S: PartialEq + Eq + Hash + Clone + Copy + Debug + Default + FromStr> AnimatedSprite<S> {
    pub fn aseprite(frames: Vec<FrameData>, texture: ArenaId<Texture>) -> Self {
        let first_frame = &frames[0];

        let sprite = Sprite::new(texture);

        let atlas = TextureAtlas::new(
            texture,
            Vec2::new(
                first_frame.source_size.w as f32,
                first_frame.source_size.h as f32,
            ),
            frames.len(),
            1,
        );

        let mut states: HashMap<S, AnimatedState<S>> = HashMap::new();

        for (index, frame) in frames.iter().enumerate() {
            if let Ok(state_key) = S::from_str(&frame.filename) {
                if states.contains_key(&state_key) {
                    let anim_frames = states.get_mut(&state_key).unwrap();
                    anim_frames.animation_frames_indices.push(index);
                } else {
                    states.insert(
                        state_key,
                        AnimatedState {
                            on_end_animation_state: None,
                            animation_frames_indices: vec![index],
                        },
                    );
                }
            } else {
                panic!(
                    "Cant turn filename to enum state, filename: {}",
                    frame.filename
                );
            }
        }

        Self {
            sprite,
            atlas,
            timer: Timer::from_seconds(first_frame.duration, timer::TimerMode::Repeating),
            current_state: S::default(),
            current_frame_index: 0,
            states,
        }
    }

    pub fn new(
        atlas: TextureAtlas,
        states: HashMap<S, AnimatedState<S>>,
        seconds_per_frame: f32,
    ) -> Self {
        let mut init = Self {
            sprite: Sprite::new(atlas.texture_handle),
            atlas,
            timer: Timer::from_seconds(seconds_per_frame, timer::TimerMode::Repeating),
            current_state: S::default(),
            current_frame_index: 0,
            states,
        };

        init.set_sprite_texture_rect(init.states[&init.current_state].clone());

        init
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
                    self.current_frame_index = 0;
                    if let Some(next_animation_state) = animation_state.on_end_animation_state {
                        dbg!("on_end_animation_state");
                        self.set_animation_state(next_animation_state);
                        animation_state = self.states.get(&next_animation_state).unwrap().clone();
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
