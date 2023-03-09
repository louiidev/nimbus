use crate::camera::Camera;
use crate::events::{CursorMoved, KeyboardInput, MouseButtonInput};
use crate::transform::GlobalTransform;
use crate::ui::UiHandler;
use bevy_ecs::prelude::{DetectChangesMut, EventReader};
use bevy_ecs::system::{Query, ResMut, Resource};
use glam::Vec2;
use std::collections::HashSet;
use std::hash::Hash;
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

#[derive(Resource, Default, Clone)]
pub struct InputController {
    pub screen_mouse_position: Vec2,
    pub mouse_position: Vec2,
    pub keyboards_inputs: Input<VirtualKeyCode>,
    pub mouse_button_inputs: Input<MouseButton>,
}

impl InputController {
    pub fn clear(&mut self) {
        self.keyboards_inputs.clear();
        self.mouse_button_inputs.clear();
    }
}

pub fn input_system(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut cursor_move_events: EventReader<CursorMoved>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut input_controller: ResMut<InputController>,
    mut ui_handler: ResMut<UiHandler>,
    camera_q: Query<(&mut Camera, &mut GlobalTransform)>,
) {
    input_controller.bypass_change_detection().clear();

    for event in keyboard_events.iter() {
        let KeyboardInput {
            key_code, state, ..
        } = event;
        match state {
            ElementState::Pressed => input_controller.keyboards_inputs.press(*key_code),
            ElementState::Released => input_controller.keyboards_inputs.release(*key_code),
        }
    }

    for event in mouse_button_events.iter() {
        let MouseButtonInput { button, state, .. } = event;

        match state {
            ElementState::Pressed => {
                input_controller.mouse_button_inputs.press(*button);
            }
            ElementState::Released => {
                input_controller.mouse_button_inputs.release(*button);
            }
        }
    }

    for event in cursor_move_events.iter() {
        let CursorMoved {
            position,
            window_size,
        } = event;
        let (camera, camera_transform) = camera_q.single();

        // move origin to bottom left
        let y_position = window_size.height as f64 - position.y;

        let mouse_pos = camera
            .viewport_to_world(
                camera_transform,
                Vec2::new(position.x as f32, y_position as f32),
            )
            .map(|ray| ray.origin.truncate());

        if let Some(mouse_pos) = mouse_pos {
            input_controller.mouse_position.x = mouse_pos.x as f32;
            input_controller.mouse_position.y = mouse_pos.y as f32;
        }

        input_controller.screen_mouse_position.x = position.x as f32;
        input_controller.screen_mouse_position.y = position.y as f32;
    }

    ui_handler.bypass_change_detection().input_controller = input_controller.clone();
}

#[derive(Debug, Resource, Clone)]
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
