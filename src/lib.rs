pub mod arena;
pub mod asset_loader;
pub mod camera;
pub mod components;
pub mod file_system_watcher;
pub mod input;
pub mod internal_image;
pub mod renderer;
pub mod systems;
pub mod time;
pub mod utils;
pub mod window;

#[cfg(feature = "debug-egui")]
use ::egui::FontDefinitions;
use asset_loader::AssetPipeline;
#[cfg(feature = "debug-egui")]
use egui_winit_platform::{Platform, PlatformDescriptor};
pub use glam as math;
use math::Vec2;
pub use sdl2;
use sdl2::{
    controller::Axis,
    event::{Event, WindowEvent},
    video::Window,
    Sdl,
};

#[cfg(feature = "debug-egui")]
pub use egui;
#[cfg(feature = "debug-egui")]
pub use egui_inspect;

use camera::Camera;
use glam::UVec2;
use input::InputManager;
use renderer::{ui::Ui, Renderer};
use systems::prepare_camera_buffers::prepare_camera_buffers;
use time::Time;
use window::WindowDescriptor;

pub trait Nimbus {
    fn init(&mut self, _engine: &mut Engine) {}
    fn update(&mut self, engine: &mut Engine, delta: f32);
    fn render(&mut self, renderer: &mut Renderer, delta: f32);
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::input::{InputEvent, InputState};

pub struct Engine {
    sdl: Sdl,
    window: Window,
    pub(crate) renderer: Option<Renderer>,
    pub input: InputManager,
    pub camera: Camera,
    pub window_size: UVec2,
    pub(crate) time: Time,
    pub ui: Ui,
    pub(crate) asset_pipeline: AssetPipeline,
    #[cfg(feature = "debug-egui")]
    pub egui_platform: Platform,
}

impl Engine {
    pub fn new(window_descriptor: WindowDescriptor) -> Self {
        let WindowDescriptor {
            width,
            height,
            title,
            ..
        } = window_descriptor;

        let mut input = InputManager::default();

        let sdl_context = sdl2::init().unwrap();
        let game_controller_subsystem = sdl_context.game_controller().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let available = game_controller_subsystem
            .num_joysticks()
            .map_err(|e| format!("can't enumerate joysticks: {}", e))
            .unwrap();

        println!("{} joysticks available", available);

        // Iterate over all available joysticks and look for game controllers.
        let mut controller = (0..available).find_map(|id| {
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

        input.controller = controller;

        let window = video_subsystem
            .window(title, width as u32, height as u32)
            .position_centered()
            .resizable()
            .metal_view()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        // let logical_size = LogicalSize::new(width, height);
        let window_size = window.size();
        let window_size = UVec2::new(window_size.0, window_size.1);
        let renderer = Some(pollster::block_on(Renderer::new(
            &window,
            UVec2::new(window_size.x, window_size.y),
        )));

        let camera = Camera::new_with_far(1000., window_size, 1. as _);

        #[cfg(feature = "debug-egui")]
        let egui_platform = Platform::new(PlatformDescriptor {
            physical_width: width as u32,
            physical_height: height as u32,
            scale_factor: 1.,
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        Self {
            sdl: sdl_context,
            window,
            renderer,
            input,
            camera,
            window_size,
            time: Time::default(),
            ui: Ui::new(window_size.as_vec2()),
            asset_pipeline: AssetPipeline::default(),
            #[cfg(feature = "debug-egui")]
            egui_platform,
        }
    }

    #[cfg(feature = "debug-egui")]
    pub fn egui_ctx(&mut self) -> egui::Context {
        self.egui_platform.context()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
    pub fn run<Game: Nimbus + 'static>(self, game: Game) {
        pollster::block_on(self.run_async(game));
    }

    pub fn get_render_ctx(&mut self) -> &mut Renderer {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer
        } else {
            self.ui.renderer.as_mut().unwrap()
        }
    }

    pub fn update<Game: Nimbus + 'static>(&mut self, game: &mut Game) {
        #[cfg(feature = "debug-egui")]
        {
            self.egui_platform
                .update_time(self.time.elapsed().as_secs_f64());
            self.egui_platform.begin_frame();
        }

        self.ui.renderer = self.renderer.take();
        game.update(self, self.time.delta_seconds());
        self.input.clear();
        let mut renderer = self.ui.renderer.take().unwrap();
        game.render(&mut renderer, self.time.delta_seconds());
        prepare_camera_buffers(&renderer, &mut self.camera);
        renderer.render(
            &self.camera,
            &mut self.ui,
            #[cfg(feature = "debug-egui")]
            &mut self.egui_platform,
            &self.window,
        );
        self.renderer = Some(renderer);
        self.time.update();
        self.watch_change();
    }

    pub async fn run_async<Game: Nimbus + 'static>(mut self, mut game: Game) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            }
        }

        game.init(&mut self);

        let mut event_pump = self
            .sdl
            .event_pump()
            .expect("Could not create sdl event pump");
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Window {
                        window_id,
                        win_event: WindowEvent::SizeChanged(width, height),
                        ..
                    } if window_id == self.window.id() => {
                        let window_size = UVec2::new(width as u32, height as u32);
                        self.window_size = window_size;
                        self.renderer.as_mut().unwrap().resize(window_size);
                        self.ui.resize(window_size.as_vec2());
                    }
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => self.input.update_keyboard_input(InputEvent {
                        state: InputState::Pressed,
                        value: keycode,
                    }),
                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => self.input.update_keyboard_input(InputEvent {
                        state: InputState::Released,
                        value: keycode,
                    }),
                    Event::MouseButtonDown { mouse_btn, .. } => {
                        self.input.update_mouse_input(InputEvent {
                            state: InputState::Pressed,
                            value: mouse_btn,
                        });
                    }

                    Event::MouseButtonUp { mouse_btn, .. } => {
                        self.input.update_mouse_input(InputEvent {
                            state: InputState::Released,
                            value: mouse_btn,
                        });
                    }

                    Event::MouseMotion { x, y, .. } => self.input.update_cursor_position(
                        (x as f32, y as f32),
                        self.window_size,
                        &self.camera,
                    ),

                    Event::ControllerAxisMotion {
                        axis: Axis::TriggerLeft,
                        value: val,
                        ..
                    } => {
                        dbg!("left trigger", val);
                    }
                    Event::ControllerAxisMotion {
                        axis: Axis::TriggerRight,
                        value: val,
                        ..
                    } => {
                        dbg!("right trigger", val);
                    }
                    Event::ControllerAxisMotion { axis, value, .. } => {
                        // Axis motion is an absolute value in the range
                        // [-32768, 32767]. Let's simulate a very rough dead
                        // zone to ignore spurious events.
                        let dead_zone = 10_000;
                        if value > dead_zone || value < -dead_zone {
                            self.input.update_axis(axis, value);
                        } else {
                            self.input.update_axis(axis, 0);
                        }
                    }
                    Event::ControllerButtonDown { button, .. } => {
                        self.input.update_gamepad_button(InputEvent {
                            state: InputState::Pressed,
                            value: button,
                        });
                    }
                    Event::ControllerButtonUp { button, .. } => {
                        self.input.update_gamepad_button(InputEvent {
                            state: InputState::Released,
                            value: button,
                        })
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

    pub fn get_viewport(&self) -> Vec2 {
        self.renderer.as_ref().unwrap().get_viewport()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(WindowDescriptor {
            width: 1280.,
            height: 720.,
            title: "Default Engine setup",
            resizable: true,
        })
    }
}
