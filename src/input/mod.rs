use glam::{UVec2, Vec2};
use sdl2::{keyboard::Keycode, mouse::MouseButton};
use std::{collections::HashSet, hash::Hash};

use crate::camera::Camera;

#[derive(Default)]
pub struct InputManager {
    pub screen_mouse_position: Vec2,
    pub mouse_position: Vec2,
    pub keyboards_inputs: Input<Keycode>,
    pub mouse_button_inputs: Input<MouseButton>,
}

pub enum InputState {
    Pressed,
    Released,
}

pub struct KeyboardInput {
    pub state: InputState,
    pub key: Keycode,
}

impl InputManager {
    pub(crate) fn update_keyboard_input(&mut self, input: KeyboardInput) {
        let KeyboardInput { key, state } = input;

        match state {
            InputState::Pressed => self.keyboards_inputs.press(key),
            InputState::Released => self.keyboards_inputs.release(key),
        }
    }

    pub(crate) fn update_mouse_input(&mut self, state: InputState, button: MouseButton) {
        match state {
            InputState::Pressed => self.mouse_button_inputs.press(button),
            InputState::Released => self.mouse_button_inputs.release(button),
        }
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
    }
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
