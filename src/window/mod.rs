#[cfg(feature = "sdl")]
pub mod sdl_window;

#[cfg(feature = "winit")]
pub mod winit_window;

use crate::{Engine, Nimbus};
use glam::UVec2;

#[cfg(feature = "winit")]
pub use self::winit_window::Gamepads;
#[cfg(feature = "winit")]
pub use self::winit_window::Window;
#[cfg(feature = "winit")]
pub use winit_window as window;

#[cfg(feature = "sdl")]
pub use self::sdl_window::Gamepads;
#[cfg(feature = "sdl")]
pub use self::sdl_window::Window;
#[cfg(feature = "sdl")]
pub use sdl_window as window;

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

pub trait WindowAbstraction {
    fn new(window_descriptor: WindowDescriptor) -> Self;
    fn get_size(&self) -> UVec2;
    fn get_scale(&self) -> f32;
    fn get_controller(&self) -> Gamepads;
    fn set_cursor_visible(&self, show: bool);
}

pub trait WindowEngineAbstraction {
    fn run_event_loop<Game: Nimbus + 'static>(self, game: Game);
}

impl Engine {
    pub fn window_resized(&mut self, window_size: UVec2) {
        self.window_size = window_size;
        self.renderer
            .render_buddy
            .resize((window_size.x, window_size.y));
        // self.ui.resize(window_size.as_vec2());
    }
}
