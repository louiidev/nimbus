use std::ops::Range;

use bevy_ecs::prelude::{Bundle, Component};
use hashbrown::HashMap;

use crate::transform::{GlobalTransform, Transform};

use super::sprite::Sprite;

#[derive(Default, Debug)]
pub struct AnimateState {
    animation_time: f32,
    on_end_animation_state: Option<String>,
    animation_frames_indices: Range<usize>,
}

#[derive(Component, Default, Debug)]
pub struct AnimatedSprite {
    states: HashMap<String, AnimateState>,
}

#[test]
fn test_animation_changes() {}
