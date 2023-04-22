use glam::{UVec2, Vec2};
use sdl2::{
    controller::{Axis, Button, GameController},
    keyboard::Keycode,
    mouse::MouseButton,
};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::camera::Camera;

#[derive(Default)]
pub struct InputManager {
    pub screen_mouse_position: Vec2,
    pub mouse_position: Vec2,
    pub keyboards_inputs: Input<Keycode>,
    pub mouse_button_inputs: Input<MouseButton>,
    pub gamepad_button_inputs: Input<Button>,
    // pub key_mappings: HashMap<String, KeyMapping>,
    pub controller: Option<GameController>,
    pub axis: HashMap<Axis, f32>,
}

pub struct KeyMapping {
    keys: Vec<Keycode>,
    gamepad_button: Vec<Button>,
    gamepad_axis: Vec<Axis>,
}

pub enum InputState {
    Pressed,
    Released,
}

pub struct InputEvent<T> {
    pub state: InputState,
    pub value: T,
}

impl InputManager {
    // pub fn create_mapping(&mut self, key: impl ToString, mapping: KeyMapping) {
    //     self.key_mappings.insert(key.to_string(), mapping);
    // }

    pub(crate) fn update_gamepad_button(&mut self, input: InputEvent<Button>) {
        let InputEvent { value, state } = input;

        match state {
            InputState::Pressed => self.gamepad_button_inputs.press(value),
            InputState::Released => self.gamepad_button_inputs.release(value),
        }
    }

    pub(crate) fn update_keyboard_input(&mut self, input: InputEvent<Keycode>) {
        let InputEvent { value, state } = input;

        match state {
            InputState::Pressed => self.keyboards_inputs.press(value),
            InputState::Released => self.keyboards_inputs.release(value),
        }
    }

    pub(crate) fn update_mouse_input(&mut self, input: InputEvent<MouseButton>) {
        let InputEvent { value, state } = input;
        match state {
            InputState::Pressed => self.mouse_button_inputs.press(value),
            InputState::Released => self.mouse_button_inputs.release(value),
        }
    }

    pub(crate) fn update_axis(&mut self, axis: Axis, value: i16) {
        let mut normalized_value = value as f32 / i16::MAX as f32;

        if let Axis::LeftY = axis {
            normalized_value = -normalized_value;
        }
        if let Axis::RightY = axis {
            normalized_value = -normalized_value;
        }

        self.axis.insert(axis, normalized_value.max(-1.));
    }

    pub(crate) fn update_cursor_position(
        &mut self,
        position: (f32, f32),
        window_size: UVec2,
        camera: &Camera,
    ) {
        let y_position = window_size.y as f32 - position.1;

        let mouse_pos = camera
            .viewport_to_world(&camera.transform, Vec2::new(position.0, y_position))
            .map(|ray| ray.origin.truncate());

        if let Some(mouse_pos) = mouse_pos {
            self.mouse_position.x = mouse_pos.x;
            self.mouse_position.y = mouse_pos.y;
        }
    }

    pub fn clear(&mut self) {
        self.keyboards_inputs.clear();
        self.mouse_button_inputs.clear();
        self.gamepad_button_inputs.clear();
    }

    pub fn get_axis(&mut self, axis: Axis) -> f32 {
        self.axis.get(&axis).copied().unwrap_or(0.)
    }

    // pub fn pressed(&self, mapping_key: impl ToString) -> bool {
    //     let mapping = self
    //         .key_mappings
    //         .get(&mapping_key.to_string())
    //         .expect(&format!("Missing mapping for {}", mapping_key.to_string()));

    //     let keypress = mapping
    //         .keys
    //         .iter()
    //         .any(|key| self.keyboards_inputs.pressed(*key));
    //     let button = mapping
    //         .gamepad_button
    //         .iter()
    //         .any(|button| self.gamepad_button_inputs.pressed(*button));

    //     keypress || button
    // }

    // /// Returns `true` if the `input` has just been pressed.
    // pub fn just_pressed(&self, mapping_key: impl ToString) -> bool {
    //     let mapping = self
    //         .key_mappings
    //         .get(&mapping_key.to_string())
    //         .expect(&format!("Missing mapping for {}", mapping_key.to_string()));

    //     let keypress = mapping
    //         .keys
    //         .iter()
    //         .any(|key| self.keyboards_inputs.just_pressed(*key));
    //     let button = mapping
    //         .gamepad_button
    //         .iter()
    //         .any(|button| self.gamepad_button_inputs.just_pressed(*button));

    //     keypress || button
    // }

    // /// Returns `true` if the `input` has just been released.
    // pub fn just_released(&self, mapping_key: impl ToString) -> bool {
    //     let mapping = self
    //         .key_mappings
    //         .get(&mapping_key.to_string())
    //         .expect(&format!("Missing mapping for {}", mapping_key.to_string()));

    //     let keypress = mapping
    //         .keys
    //         .iter()
    //         .any(|key| self.keyboards_inputs.just_released(*key));
    //     let button = mapping
    //         .gamepad_button
    //         .iter()
    //         .any(|button| self.gamepad_button_inputs.just_released(*button));

    //     keypress || button
    // }
}

#[derive(Debug, Clone)]
pub struct Input<T: Copy + Eq + Hash + Send + Sync + 'static> {
    /// A collection of every button that is currently being pressed.
    pressed: HashSet<T>,
    /// A collection of every button that has just been pressed.
    just_pressed: HashSet<T>,
    /// A collection of every button that has just been released.
    just_released: HashSet<T>,
}

impl<T: Copy + Eq + Hash + Send + Sync + 'static> Default for Input<T> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
        }
    }
}

impl<T> Input<T>
where
    T: Copy + Eq + Hash + Send + Sync + 'static,
{
    /// Registers a press for the given `input`.
    pub fn press(&mut self, input: T) {
        // Returns `true` if the `input` wasn't pressed.
        if self.pressed.insert(input) {
            self.just_pressed.insert(input);
        }
    }

    /// Returns `true` if the `input` has been pressed.
    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    /// Returns `true` if the `input` has just been pressed.
    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    /// Returns `true` if the `input` has just been released.
    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }

    /// Registers a release for the given `input`.
    pub fn release(&mut self, input: T) {
        // Returns `true` if the `input` was pressed.
        if self.pressed.remove(&input) {
            self.just_released.insert(input);
        }
    }

    /// Clears the `pressed`, `just_pressed` and `just_released` data of the `input`.
    pub fn reset(&mut self, input: T) {
        self.pressed.remove(&input);
        self.just_pressed.remove(&input);
        self.just_released.remove(&input);
    }

    /// An iterator visiting every pressed input in arbitrary order.
    pub fn get_pressed(&self) -> impl ExactSizeIterator<Item = &T> {
        self.pressed.iter()
    }

    /// An iterator visiting every just pressed input in arbitrary order.
    pub fn get_just_pressed(&self) -> impl ExactSizeIterator<Item = &T> {
        self.just_pressed.iter()
    }

    /// An iterator visiting every just released input in arbitrary order.
    pub fn get_just_released(&self) -> impl ExactSizeIterator<Item = &T> {
        self.just_released.iter()
    }

    /// Clears the `just pressed` and `just released` data for every input.
    ///
    /// See also [`Input::reset_all`] for a full reset.
    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}
