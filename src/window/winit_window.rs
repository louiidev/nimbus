use gilrs::{Filter, Gilrs};
use glam::UVec2;
use winit::window::Window as ExternalWindow;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

use crate::input::InputEvent;
use crate::Engine;

use super::{WindowAbstraction, WindowDescriptor, WindowEngineAbstraction};

pub struct Window {
    pub(crate) window: ExternalWindow,
    event_loop: Option<EventLoop<()>>,
}

#[derive(Default)]
pub struct Gamepads {
    #[cfg(feature = "winit")]
    pub(crate) gilrs: Option<Gilrs>,
}

impl WindowAbstraction for Window {
    fn new(window_descriptor: WindowDescriptor) -> Self {
        let WindowDescriptor {
            width,
            height,
            title,
            ..
        } = window_descriptor;

        let logical_size = LogicalSize::new(width, height);
        let window_builder = WindowBuilder::new()
            .with_inner_size(logical_size)
            .with_title(title);

        let event_loop = EventLoop::new();

        let window = window_builder.build(&event_loop).unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            use winit::platform::web::WindowExtWebSys;

            use log::info;
            use log::Level;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let body = doc.body().expect("document expect to have have a body");
                    let canvas = web_sys::Element::from(window.canvas());
                    body.append_child(&canvas).ok().expect("Cant append");

                    // Request fullscreen, if denied, continue as normal
                    match canvas.request_fullscreen() {
                        Ok(_) => Some(()),
                        Err(_) => Some(()),
                    }
                })
                .unwrap();
        }

        Self {
            window,
            event_loop: Some(event_loop),
        }
    }

    fn get_size(&self) -> glam::UVec2 {
        let size = self.window.inner_size();
        UVec2::new(size.width, size.height)
    }

    fn get_scale(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    fn get_controller(&self) -> Gamepads {
        Gamepads {
            gilrs: Some(Gilrs::new().unwrap()),
        }
    }
}

impl WindowEngineAbstraction for Engine {
    fn run_event_loop<Game: crate::Nimbus + 'static>(mut self, mut game: Game) {
        use gilrs::{ev::filter::axis_dpad_to_button, EventType};
        use winit::{
            event::{Event, WindowEvent},
            event_loop::ControlFlow,
        };

        use crate::input::{Axis, Input, InputState};

        game.init(&mut self);

        let event_loop = self.window.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| {
            let current_window_id = self.window.window.id();
            *control_flow = ControlFlow::Wait;

            #[cfg(feature = "debug-egui")]
            self.egui_platform.handle_event(&event);

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == current_window_id => match event {
                    WindowEvent::CloseRequested {} => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        let window_size = UVec2::new(physical_size.width, physical_size.height);
                        self.window_resized(window_size);
                    }
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        scale_factor,
                    } => {
                        let window_size = UVec2::new(new_inner_size.width, new_inner_size.height);

                        self.window_resized(window_size);
                    }
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        if let Some(value) = input.virtual_keycode {
                            let input_event = InputEvent {
                                state: input.state.into(),
                                value,
                            };

                            self.input.update_input_state(input_event);
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let state = *state;
                        let button = *button;
                        let input_event = InputEvent {
                            state: state.into(),
                            value: button,
                        };

                        self.input.update_input_state(input_event);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.input.update_cursor_position(
                            (position.x as f32, position.y as f32),
                            self.window_size,
                            &self.camera,
                        );
                    }
                    _ => {}
                },

                Event::RedrawEventsCleared => *control_flow = ControlFlow::Poll,
                Event::MainEventsCleared => self.update(&mut game),
                _ => {}
            }

            if let Some(mut gilrs) = self.input.controllers.gilrs.take() {
                while let Some(gilrs_event) = gilrs
                    .next_event()
                    .filter_ev(&axis_dpad_to_button, &mut gilrs)
                {
                    match gilrs_event.event {
                        // EventType::Connected => {
                        //     let pad = gilrs.gamepad(gilrs_event.id);
                        //     let info = GamepadInfo {
                        //         name: pad.name().into(),
                        //     };
                        // }
                        // EventType::Disconnected => {
                        //    let pad = gilrs.gamepad(gilrs_event.id);
                        // }
                        // ),
                        EventType::ButtonChanged(gilrs_button, raw_value, _) => {
                            const THRESHOLD: f32 = 0.75;
                            // const release_threshold: f32 = 0.65;
                            let button: Result<Input, _> = gilrs_button.try_into();

                            let state = if raw_value >= THRESHOLD {
                                InputState::Pressed
                            } else {
                                InputState::Released
                            };

                            if let Ok(value) = button {
                                self.input.update_input_state(InputEvent { state, value });
                            }
                        }
                        EventType::AxisChanged(gilrs_axis, raw_value, _) => {
                            let axis: Result<Axis, _> = gilrs_axis.try_into();
                            if let Ok(axis) = axis {
                                self.input.update_axis(axis, raw_value)
                            }
                        }
                        _ => (),
                    }
                }

                self.input.controllers.gilrs = Some(gilrs);
            }
        });
    }
}
