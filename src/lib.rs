pub mod arena;
pub mod asset_loader;
pub mod audio;
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
use audio::Audio;
#[cfg(feature = "debug-egui")]
use egui_winit_platform::{Platform, PlatformDescriptor};
pub use glam as math;
use math::Vec2;

#[cfg(feature = "debug-egui")]
pub use egui;
#[cfg(feature = "debug-egui")]
pub use egui_inspect;

use camera::Camera;
use glam::UVec2;
use input::InputManager;
use renderer::{ui::Ui, Renderer};
use time::Time;
use window::WindowDescriptor;

pub trait Nimbus {
    fn init(&mut self, _engine: &mut Engine) {}
    fn update(&mut self, engine: &mut Engine, delta: f32);
    fn render(&mut self, renderer: &mut Renderer, delta: f32);
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{camera::ScalingMode, window::Window};

pub struct Engine {
    pub camera: Camera,
    window: Window,
    pub(crate) renderer: Option<Renderer>,
    pub input: InputManager,
    pub window_size: UVec2,
    pub(crate) time: Time,
    pub ui: Ui,
    pub(crate) asset_pipeline: AssetPipeline,
    pub audio: Audio,
    #[cfg(feature = "debug-egui")]
    pub egui_platform: Platform,
}

impl Engine {
    pub fn new(window_descriptor: WindowDescriptor) -> Self {
        let window = Window::new(window_descriptor);

        let input = InputManager {
            controllers: window.get_controller(),
            ..Default::default()
        };

        // let logical_size = LogicalSize::new(width, height);
        let window_size = window.size();

        let mut camera = Camera::new_with_far(1000., window_size, 1);

        if let Some(size) = window_descriptor.render_resolution {
            camera.projection.scaling_mode = ScalingMode::AutoMax {
                max_width: size.x as f32,
                max_height: size.y as f32,
            };
            camera.resize(window_size);
        }

        let renderer = Some(pollster::block_on(Renderer::new(
            &window,
            UVec2::new(window_size.x, window_size.y),
        )));

        #[cfg(feature = "debug-egui")]
        let egui_platform = Platform::new(PlatformDescriptor {
            physical_width: width as u32,
            physical_height: height as u32,
            scale_factor: 1.,
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        Self {
            camera,
            window,
            renderer,
            input,
            window_size,
            time: Time::default(),
            ui: Ui::new(window_size.as_vec2()),
            asset_pipeline: AssetPipeline::default(),
            audio: Audio::default(),
            #[cfg(feature = "debug-egui")]
            egui_platform,
        }
    }

    #[cfg(feature = "debug-egui")]
    pub fn egui_ctx(&mut self) -> egui::Context {
        self.egui_platform.context()
    }

    pub fn run<Game: Nimbus + 'static>(self, game: Game) {
        pollster::block_on(self.run_event_loop_async(game));
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

    pub fn get_viewport(&self) -> Vec2 {
        self.renderer.as_ref().unwrap().get_viewport()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
