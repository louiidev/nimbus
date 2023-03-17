use std::ops::{Deref, DerefMut, Range};

use bevy_ecs::{
    prelude::{Bundle, Component},
    system::{Query, Res},
};
use hashbrown::HashMap;

use crate::{
    resources::utils::Assets,
    texture_atlas::TextureAtlas,
    time::Time,
    transform::{GlobalTransform, Transform},
};

use super::{sprite::Sprite, timer::Timer};

#[derive(Default, Debug, Clone)]
pub struct AnimateState {
    pub on_end_animation_state: Option<String>,
    pub animation_frames_indices: Range<usize>,
}

#[derive(Component, Default, Debug)]
pub struct AnimatedSprite {
    pub states: HashMap<String, AnimateState>,
    pub texture_atlas_id: uuid::Uuid,
    pub current_state: Option<String>,
    current_frame: usize,
}

impl AnimatedSprite {
    pub fn new(
        states: HashMap<String, AnimateState>,
        texture_atlas_id: uuid::Uuid,
        current_state: Option<String>,
        current_frame: usize,
    ) -> Self {
        Self {
            states,
            texture_atlas_id,
            current_state,
            current_frame,
        }
    }

    pub fn set_animation_state(&mut self, new_state: &str) {
        let new_state_string = Some(new_state.to_string());
        if new_state_string != self.current_state {
            let state = self.states.get(new_state).unwrap();
            self.current_state = new_state_string;
            self.current_frame = state.animation_frames_indices.start;
        }
    }
}

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

pub fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut AnimationTimer, &mut AnimatedSprite, &mut Sprite)>,
) {
    for (mut timer, mut animation, mut sprite) in &mut query {
        if let Some(current_state) = animation.current_state.clone() {
            if let Some(animation_state) = animation.states.get(&current_state).cloned() {
                timer.0.tick(time.delta());
                if timer.0.just_finished() {
                    let last_frame = animation_state.animation_frames_indices.end;

                    if animation.current_frame == last_frame {
                        if let Some(next_animation_state) = animation_state.on_end_animation_state {
                            animation.set_animation_state(&next_animation_state);
                        } else {
                            animation.current_frame =
                                animation_state.animation_frames_indices.start;
                        }
                    } else {
                        animation.current_frame += 1;
                    }

                    // TODO maybe this should be the sprite texture id?
                    let sprite_rect = texture_atlases
                        .get(&animation.texture_atlas_id)
                        .unwrap()
                        .textures
                        .get(animation.current_frame)
                        .copied();

                    sprite.texture_rect = sprite_rect;
                }
            }
        }
    }
}
