use glam::UVec2;
use sdl2::{
    controller::GameController,
    event::{Event, WindowEvent},
    video::Window as ExternalWindow,
    Sdl,
};

use crate::{input::InputEvent, Engine, Nimbus};

use super::{WindowAbstraction, WindowDescriptor, WindowEngineAbstraction};

pub struct Window {
    pub window: ExternalWindow,
    sdl_context: Sdl,
}

#[derive(Default)]
pub struct Gamepads {
    #[cfg(feature = "sdl")]
    pub(crate) controller: Option<GameController>,
}

impl WindowAbstraction for Window {
    fn new(window_descriptor: WindowDescriptor) -> Self {
        let WindowDescriptor {
            width,
            height,
            title,
            ..
        } = window_descriptor;

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(title, width as u32, height as u32)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        Self {
            window,
            sdl_context,
        }
    }

    fn get_size(&self) -> glam::UVec2 {
        let size = self.window.size();
        UVec2::new(size.0, size.1)
    }

    fn get_scale(&self) -> f32 {
        1f32
    }

    fn get_controller(&self) -> Gamepads {
        #[cfg(feature = "sdl")]
        {
            let game_controller_subsystem = self.sdl_context.game_controller().unwrap();

            let available = game_controller_subsystem
                .num_joysticks()
                .map_err(|e| format!("can't enumerate joysticks: {}", e))
                .unwrap();

            println!("{} joysticks available", available);

            // Iterate over all available joysticks and look for game controllers.
            let controller = (0..available).find_map(|id| {
                if !game_controller_subsystem.is_game_controller(id) {
                    println!("{} is not a game controller", id);
                    return None;
                }

                println!("Attempting to open controller {}", id);

                match game_controller_subsystem.open(id) {
                    Ok(c) => {
                        // We managed to find and open a game controller,
                        // exit the loop
                        println!("Success: opened \"{}\"", c.name());
                        Some(c)
                    }
                    Err(e) => {
                        println!("failed: {:?}", e);
                        None
                    }
                }
            });

            Gamepads { controller }
        }
    }
}

impl WindowEngineAbstraction for Engine {
    fn run_event_loop<Game: Nimbus + 'static>(mut self, mut game: Game) {
        use crate::input::{Axis, Input, InputState};

        game.init(&mut self);

        let mut event_pump = self
            .window
            .sdl_context
            .event_pump()
            .expect("Could not create sdl event pump");
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Window {
                        window_id,
                        win_event: WindowEvent::SizeChanged(width, height),
                        ..
                    } if window_id == self.window.window.id() => {
                        let window_size = UVec2::new(width as u32, height as u32);
                        self.window_resized(window_size);
                    }
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        let keycode: Result<Input, _> = keycode.try_into();

                        if let Ok(keycode) = keycode {
                            self.input.update_input_state(InputEvent {
                                state: InputState::Pressed,
                                value: keycode,
                            })
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => {
                        let keycode: Result<Input, _> = keycode.try_into();

                        if let Ok(keycode) = keycode {
                            self.input.update_input_state(InputEvent {
                                state: InputState::Released,
                                value: keycode,
                            })
                        }
                    }
                    Event::MouseButtonDown { mouse_btn, .. } => {
                        let mouse_btn: Result<Input, _> = mouse_btn.try_into();

                        if let Ok(mouse_btn) = mouse_btn {
                            self.input.update_input_state(InputEvent {
                                state: InputState::Pressed,
                                value: mouse_btn,
                            });
                        }
                    }

                    Event::MouseButtonUp { mouse_btn, .. } => {
                        let mouse_btn: Result<Input, _> = mouse_btn.try_into();

                        if let Ok(mouse_btn) = mouse_btn {
                            self.input.update_input_state(InputEvent {
                                state: InputState::Released,
                                value: mouse_btn,
                            });
                        }
                    }

                    Event::MouseMotion { x, y, .. } => {
                        self.input.update_cursor_position(
                            Vec2::new(x as f32, y as f32),
                            self.window_size,
                        );
                    }

                    Event::ControllerAxisMotion {
                        axis: sdl2::controller::Axis::TriggerLeft,
                        value,
                        ..
                    } => {
                        let dead_zone = 10_000;

                        let state = if value > dead_zone || value < -dead_zone {
                            InputState::Pressed
                        } else {
                            InputState::Released
                        };

                        self.input.update_input_state(InputEvent {
                            state,
                            value: Input::GamepadLeftTrigger,
                        });
                    }
                    Event::ControllerAxisMotion {
                        axis: sdl2::controller::Axis::TriggerRight,
                        value,
                        ..
                    } => {
                        let dead_zone = 10_000;

                        let state = if value > dead_zone || value < -dead_zone {
                            InputState::Pressed
                        } else {
                            InputState::Released
                        };

                        self.input.update_input_state(InputEvent {
                            state,
                            value: Input::GamepadRightTrigger,
                        });
                    }
                    Event::ControllerAxisMotion { axis, value, .. } => {
                        let axis: Result<Axis, _> = axis.try_into();

                        if let Ok(axis) = axis {
                            // Axis motion is an absolute value in the range
                            // [-32768, 32767]. Let's simulate a very rough dead
                            // zone to ignore spurious events.
                            let dead_zone = 10_000;
                            if value > dead_zone || value < -dead_zone {
                                let mut normalized_value = value as f32 / i16::MAX as f32;

                                if let Axis::LeftY = axis {
                                    normalized_value = -normalized_value;
                                }
                                if let Axis::RightY = axis {
                                    normalized_value = -normalized_value;
                                }

                                normalized_value.max(-1.);
                                self.input.update_axis(axis, normalized_value);
                            } else {
                                self.input.update_axis(axis, 0f32);
                            }
                        }
                    }
                    Event::ControllerButtonDown { button, .. } => {
                        let btn: Result<Input, _> = button.try_into();

                        if let Ok(btn) = btn {
                            self.input.update_input_state(InputEvent {
                                state: InputState::Pressed,
                                value: btn,
                            });
                        }
                    }
                    Event::ControllerButtonUp { button, .. } => {
                        let btn: Result<Input, _> = button.try_into();

                        if let Ok(btn) = btn {
                            self.input.update_input_state(InputEvent {
                                state: InputState::Released,
                                value: btn,
                            });
                        }
                    }

                    Event::Quit { .. } => {
                        break 'running;
                    }

                    _e => {}
                }
            }

            self.update(&mut game);
        }
    }
}
