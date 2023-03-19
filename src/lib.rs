pub mod areana;
pub mod camera;
pub mod components;
pub mod input;
pub mod renderer;
pub mod systems;
pub mod utils;
pub mod window;

use camera::Camera;
use components::time::Time;
use glam::UVec2;
use input::InputManager;
use renderer::Renderer;
use systems::{
    post_render::post_render, prepare_camera_buffers::prepare_camera_buffers,
    prepare_render::prepare_mesh2d_for_batching,
};
use window::WindowDescriptor;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub trait Guacamole {
    fn init(&mut self, engine: &mut Engine) {}
    fn update(&mut self, engine: &mut Engine) {}
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct Engine {
    window: Window,
    event_loop: Option<EventLoop<()>>,
    pub renderer: Renderer,
    pub input: InputManager,
    pub camera: Camera,
    pub window_size: UVec2,
    pub time: Time,
}

// impl Default for Engine {
//     fn default() -> Self {
//         let event_loop = EventLoop::new();
//         let window = WindowBuilder::new().build(&event_loop).unwrap();
//         Self {
//             event_loop: Some(event_loop),
//             window,
//             world: World::default(),
//             renderer: Renderer::default(),
//         }
//     }
// }

impl Engine {
    pub fn new(window_descriptor: WindowDescriptor) -> Self {
        let WindowDescriptor {
            width,
            height,
            title,
            ..
        } = window_descriptor;

        let logical_size = LogicalSize::new(width, height);

        let event_loop = EventLoop::new();

        let mut window_builder = WindowBuilder::new()
            .with_inner_size(logical_size)
            .with_title(title);

        let window = window_builder.build(&event_loop).unwrap();
        let window_size = window.inner_size();
        let window_size = UVec2::new(window_size.width, window_size.height);
        dbg!(window_size);
        let renderer = pollster::block_on(Renderer::new(
            &window,
            UVec2::new(window_size.x, window_size.y),
        ));

        let camera = Camera::new_with_far(1000., window_size, window.scale_factor() as _);
        Self {
            event_loop: Some(event_loop),
            window,
            renderer,
            input: InputManager::default(),
            camera,
            window_size,
            time: Time::default(),
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
    pub fn run<Game: Guacamole + 'static>(self, game: Game) {
        pollster::block_on(self.run_async(game));
    }

    pub fn update<Game: Guacamole + 'static>(&mut self, game: &mut Game) {
        game.update(self);
        prepare_camera_buffers(&self.renderer, &mut self.camera);
        self.renderer.render(&self.camera);
        post_render(&mut self.renderer);
        self.time.update();
    }

    pub async fn run_async<Game: Guacamole + 'static>(mut self, mut game: Game) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            window.set_inner_size(PhysicalSize::new(450, 400));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let event_loop = self.event_loop.take().unwrap();

        game.init(&mut self);

        event_loop.run(move |event, _, control_flow| {
            let window = &self.window;
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested {} => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        let window_size = UVec2::new(physical_size.width, physical_size.height);
                        self.window_size = window_size;
                        self.renderer.resize(window_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        let window_size = UVec2::new(new_inner_size.width, new_inner_size.height);
                        self.renderer.resize(window_size);
                    }
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        self.input.update_keyboard_input(input);
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        self.input.update_mouse_input(state, button);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.input
                            .update_cursor_position(position, self.window_size, &self.camera);
                    }
                    _ => {}
                },

                Event::RedrawEventsCleared => *control_flow = ControlFlow::Poll,
                Event::MainEventsCleared => self.update(&mut game),
                _ => {}
            }
        });
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
