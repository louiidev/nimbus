pub mod arena;
pub mod asset_loader;
pub mod audio;
pub mod components;
#[cfg(feature = "hot-reloading")]
pub mod file_system_watcher;
pub mod input;
pub mod internal_image;
pub mod renderer;
pub mod time;
pub mod utils;
pub mod window;

pub use crate::arena::*;
pub use crate::rect::*;
pub use crate::renderer::texture::*;
pub use crate::sprite::*;
pub use crate::text::*;
pub use crate::transform::*;
use asset_loader::AssetPipeline;
use audio::Audio;
use components::color::Color;
#[cfg(feature = "egui")]
pub use egui;
use egui::Ui;
use egui_inspect::EguiInspect;
#[cfg(feature = "egui")]
use egui_winit_platform::{Platform, PlatformDescriptor};
pub use glam as math;
use glam::UVec2;
use input::InputManager;
use renderer::camera::CameraController;
pub use renderer::*;
pub use wgpu;
pub use yakui;
// use old_renderer::{ui::Ui, Renderer};
use renderer::{camera::Camera, Renderer};
use time::Time;
use window::{WindowAbstraction, WindowDescriptor, WindowEngineAbstraction};

pub trait Nimbus {
    fn init(&mut self, _engine: &mut Engine) {}
    fn update(&mut self, engine: &mut Engine, delta: f32);
    fn render(&mut self, renderer: &mut Renderer, delta: f32);
    fn inspector(&mut self, engine: &Engine, ui: &mut Ui) {}
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::window::Window;

pub struct EditorState {
    paused: bool,
    delta_time_multiplier: f32,
    pub inspectable: Vec<Box<dyn FnMut(&mut egui::Ui)>>,
}

impl Default for EditorState {
    fn default() -> Self {
        EditorState {
            paused: false,
            delta_time_multiplier: 1f32,
            inspectable: Vec::default(),
        }
    }
}

pub struct Engine {
    pub camera: Camera,
    window: Window,
    pub renderer: Renderer,
    pub input: InputManager,
    pub window_size: UVec2,
    pub time: Time,
    // pub ui: Ui,
    pub(crate) asset_pipeline: AssetPipeline,
    pub audio: Audio,
    #[cfg(feature = "egui")]
    pub egui_platform: Platform,
    pub editor_state: EditorState,
    #[cfg(debug_assertions)]
    pub camera_controller: CameraController,
    pub yakui: yakui::Yakui,
}

impl Engine {
    pub fn new(window_descriptor: WindowDescriptor) -> Self {
        let window = Window::new(window_descriptor);

        let input = InputManager {
            controllers: window.get_controller(),
            ..Default::default()
        };

        let window_size = window.get_size();

        let mut camera = Camera::default();

        let renderer = pollster::block_on(Renderer::new(
            &window.window,
            (window_size.x, window_size.y),
        ))
        .unwrap();

        #[cfg(feature = "egui")]
        let egui_platform = Platform::new(PlatformDescriptor {
            physical_width: window_size.x as u32,
            physical_height: window_size.y as u32,
            scale_factor: 1.,
            ..Default::default()
        });

        let yakui = yakui::Yakui::new();

        Self {
            yakui,
            camera,
            window,
            renderer,
            input,
            window_size,
            time: Time::default(),
            // ui: Ui::new(window_size.as_vec2()),
            asset_pipeline: AssetPipeline::default(),
            audio: Audio::default(),
            #[cfg(feature = "egui")]
            egui_platform,
            editor_state: EditorState::default(),
            #[cfg(debug_assertions)]
            camera_controller: CameraController::new(10.0, 0.4),
        }
    }

    #[cfg(feature = "egui")]
    pub fn egui_ctx(&mut self) -> egui::Context {
        self.egui_platform.context()
    }

    pub fn run<Game: Nimbus + 'static>(self, game: Game) {
        self.run_event_loop(game);
    }

    pub fn update<Game: Nimbus + 'static>(&mut self, game: &mut Game) {
        #[cfg(feature = "egui")]
        {
            self.egui_platform
                .update_time(self.time.elapsed().as_secs_f64());
            self.egui_platform.begin_frame();
        }

        #[cfg(debug_assertions)]
        {
            // self.update_camera(self.time.delta_seconds());
        }

        #[cfg(all(debug_assertions, feature = "egui"))]
        {
            egui::Window::new("Editor")
                .default_width(320.)
                .default_open(false)
                .show(&self.egui_ctx(), |ui| {
                    let delta_seconds = self.time.raw_delta_seconds_f64();
                    ui.label(format!("Frame time: {}", (delta_seconds * 1000.0) as i32));
                    ui.label(format!("FPS: {}", (1. / delta_seconds) as i32));
                    ui.checkbox(&mut self.editor_state.paused, "Pause Game");
                    ui.add(
                        egui::Slider::new(&mut self.editor_state.delta_time_multiplier, 0.0..=2.0)
                            .text("Delta time multiplier"),
                    );
                    self.camera.inspect_mut("test", ui);
                    game.inspector(self, ui);
                });
        }

        let delta = self.time.delta_seconds() * self.editor_state.delta_time_multiplier;
        self.yakui.start();
        if !self.editor_state.paused {
            game.update(self, delta);
        }
        self.clear_inputs();

        #[cfg(feature = "egui")]
        let full_output = self.egui_platform.end_frame(Some(&self.window.window));
        #[cfg(feature = "egui")]
        let paint_jobs = self.egui_platform.context().tessellate(full_output.shapes);

        let mut ctx = self.renderer.begin();
        game.render(&mut self.renderer, delta);
        self.yakui.finish();
        self.renderer.render(
            &mut ctx,
            Color::hex("#6b6ab3").unwrap().as_rgba_linear(),
            &self.camera,
            &mut self.yakui,
        );

        self.renderer
            .render_egui(&mut ctx, &full_output.textures_delta, &paint_jobs);
        self.renderer.end_frame(ctx);
        self.renderer.end_egui(full_output.textures_delta);
        self.time.update();
        self.watch_change();
    }

    pub fn enable_wasm_logs() {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            }
        }
    }

    pub fn get_viewport(&self) -> (u32, u32) {
        self.renderer.get_viewport_size()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
