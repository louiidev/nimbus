#[cfg(feature = "sdl")]
use sdl2::{
    controller::GameController,
    event::{Event, WindowEvent},
    raw_window_handle,
    video::Window as ExternalWindow,
    Sdl,
};

#[cfg(feature = "winit")]
use gilrs::Gilrs;
#[cfg(feature = "winit")]
use winit::window::Window as ExternalWindow;
#[cfg(feature = "winit")]
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

use crate::{input::InputEvent, Engine, Nimbus};
use glam::UVec2;

#[derive(Clone, Copy)]
pub struct WindowDescriptor<'a> {
    pub width: f32,
    pub height: f32,
    pub title: &'a str,
    /// ## Platform-specific
    /// - iOS / Android / Web: Unsupported.
    pub resizable: bool,
    pub render_resolution: Option<UVec2>,
}

impl<'a> Default for WindowDescriptor<'a> {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 720.0,
            title: "Nimbus Engine",
            resizable: false,
            render_resolution: None,
        }
    }
}

pub struct Window {
    window: ExternalWindow,
    #[cfg(feature = "sdl")]
    sdl_context: Sdl,
    #[cfg(feature = "winit")]
    event_loop: Option<EventLoop<()>>,
}

#[derive(Default)]
pub struct Gamepads {
    #[cfg(feature = "sdl")]
    pub(crate) controller: Option<GameController>,
    #[cfg(feature = "winit")]
    pub(crate) gilrs: Option<Gilrs>,
}

impl Window {
    pub(crate) fn new(window_descriptor: WindowDescriptor) -> Self {
        let WindowDescriptor {
            width,
            height,
            title,
            ..
        } = window_descriptor;

        #[cfg(feature = "sdl")]
        {
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

        #[cfg(feature = "winit")]
        {
            let logical_size = LogicalSize::new(width, height);
            let window_builder = WindowBuilder::new()
                .with_inner_size(logical_size)
                .with_title(title);

            let event_loop = EventLoop::new();

            let window = window_builder.build(&event_loop).unwrap();

            Self {
                window,
                event_loop: Some(event_loop),
            }
        }
    }

    pub(crate) async fn create_surface_adapater(
        &self,
    ) -> (wgpu::Instance, wgpu::Surface, wgpu::Adapter) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&self.window).unwrap() };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        (instance, surface, adapter)
    }

    pub fn size(&self) -> UVec2 {
        #[cfg(feature = "sdl")]
        {
            let size = self.window.size();
            UVec2::new(size.0, size.1)
        }

        #[cfg(feature = "winit")]
        {
            let size = self.window.inner_size();
            UVec2::new(size.width, size.height)
        }
    }

    pub fn get_scale(&self) -> f32 {
        #[cfg(feature = "sdl")]
        {
            // @TODO: work out how to get scale from sdl
            1f32
        }

        #[cfg(feature = "winit")]
        {
            self.window.scale_factor() as f32
        }
    }

    pub(crate) fn get_controller(&self) -> Gamepads {
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

        #[cfg(feature = "winit")]
        {
            Gamepads {
                gilrs: Some(Gilrs::new().unwrap()),
            }
        }
    }
}

impl Engine {
    pub fn window_resized(&mut self, window_size: UVec2) {
        self.window_size = window_size;
        self.renderer.as_mut().unwrap().resize(window_size);
        self.ui.resize(window_size.as_vec2());
        self.camera.resize(window_size);
    }

    #[cfg(feature = "winit")]
    pub async fn run_event_loop_async<Game: Nimbus + 'static>(mut self, mut game: Game) {
        use gilrs::{ev::filter::axis_dpad_to_button, EventType, Filter};
        use winit::{
            event::{Event, WindowEvent},
            event_loop::ControlFlow,
        };

        use crate::input::{Axis, Input, InputState};

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            }
        }

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
    #[cfg(feature = "sdl")]
    pub async fn run_event_loop_async<Game: Nimbus + 'static>(mut self, mut game: Game) {
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
                            (x as f32, y as f32),
                            self.window_size,
                            &self.camera,
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
