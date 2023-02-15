use bevy_ecs::{
    prelude::{Bundle, Events},
    schedule::{IntoSystemDescriptor, Schedule, Stage, StageLabel, SystemStage},
    system::{Res, Resource},
    world::{FromWorld, World},
};

use events::{CursorMoved, KeyboardInput, MouseButtonInput, WindowCreated, WindowResized};
use font::FontAtlasSet;
use internal_image::{Image, DEFAULT_TEXTURE_FORMAT};
use renderer::{render_system, renderable::RenderCache, texture::Texture, Renderer};

use resources::{
    inputs::{input_system, InputController},
    utils::Assets,
};
use texture_atlas::TextureAtlas;
use time::Time;
use transform::transform_propagate_system;
use ui::UiHandler;
use wgpu::{Extent3d, TextureDimension};
use window::{create_window, WindowDescriptor};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub const DEFAULT_TEXTURE_ID: uuid::Uuid =
    uuid::Uuid::from_u128(0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8u128);

pub mod camera;
pub mod color;
pub mod components;
pub mod events;
#[path = "image.rs"]
pub mod internal_image;
pub mod rect;
pub mod renderer;
pub mod resources;
pub mod texture_atlas;
pub mod time;
pub mod transform;
pub mod ui;
pub mod window;

pub mod font;
pub mod loaders;
pub mod utils;

pub use bevy_ecs as ecs;
pub use glam as math;
pub use winit;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum CoreStage {
    /// The [`Stage`](bevy_ecs::schedule::Stage) that runs before all other app stages.
    First,
    /// The [`Stage`](bevy_ecs::schedule::Stage) that runs before [`CoreStage::Update`].
    PreUpdate,
    /// The [`Stage`](bevy_ecs::schedule::Stage) responsible for doing most app logic. Systems should be registered here by default.
    Update,
    /// The [`Stage`](bevy_ecs::schedule::Stage) that runs after [`CoreStage::Update`].
    PostUpdate,
    PrepareRenderer,
    Render,
    /// Cleanup render cache that runs after [`CoreStage::Render`].
    PostRender,
}

pub struct App {
    pub world: World,
    schedule: Schedule,
    event_loop: Option<EventLoop<()>>,
    window: Window,
}

impl Default for App {
    fn default() -> Self {
        let mut world = World::default();
        let (window, event_loop) = create_window(WindowDescriptor::default());
        let default_font_id = uuid::Uuid::new_v4();
        let renderer = pollster::block_on(Renderer::new(&window, default_font_id));
        world.insert_resource(renderer);
        let window_size = window.inner_size();
        world.send_event(WindowCreated {
            width: window_size.width as f32,
            height: window_size.height as f32,
        });

        App {
            schedule: Schedule::default(),
            event_loop: Some(event_loop),
            window,
            world,
        }
    }
}

impl App {
    fn add_default_stages(&mut self) {
        self.schedule
            .add_stage(CoreStage::First, SystemStage::parallel())
            .add_stage(CoreStage::PreUpdate, SystemStage::parallel())
            .add_stage(CoreStage::Update, SystemStage::parallel())
            .add_stage(CoreStage::PostUpdate, SystemStage::parallel())
            .add_stage(CoreStage::PrepareRenderer, SystemStage::parallel())
            .add_stage(CoreStage::Render, SystemStage::parallel())
            .add_stage(CoreStage::PostRender, SystemStage::parallel());

        self.add_event::<WindowResized>()
            .add_event::<WindowCreated>()
            .add_event::<KeyboardInput>()
            .add_event::<CursorMoved>()
            .add_event::<MouseButtonInput>();
    }

    fn add_assets(mut self) -> Self {
        self.add_asset::<Texture>();
        self.add_asset::<TextureAtlas>();
        self.add_asset::<Image>();
        self.add_asset::<FontAtlasSet>();

        let image = Image::new_fill(
            Extent3d::default(),
            TextureDimension::D2,
            &[255u8; 4],
            DEFAULT_TEXTURE_FORMAT,
        );
        self.load_texture_with_id_image(image, DEFAULT_TEXTURE_ID);

        self
    }

    fn add_default_system_resources(mut self) -> Self {
        self.add_default_stages();
        self.init_resource::<InputController>();

        self.init_resource::<Time>();

        self.init_resource::<RenderCache>();

        self.add_system_to_stage(render_system, CoreStage::Render)
            .add_system_to_stage(transform_propagate_system, CoreStage::PostUpdate)
            .add_system_to_stage(crate::camera::camera_system, CoreStage::PostUpdate)
            .add_system_to_stage(input_system, CoreStage::PreUpdate)
            .init_2d_renderer()
            .init_ui_renderer()
    }

    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    pub fn add_asset<T: Send + Sync + 'static>(&mut self) {
        self.world.insert_resource::<Assets<T>>(Assets::new());
    }

    pub fn add_event<T>(&mut self) -> &mut Self
    where
        T: crate::events::Event,
    {
        if !self.world.contains_resource::<Events<T>>() {
            self.init_resource::<Events<T>>()
                .schedule
                .add_system_to_stage(CoreStage::First, Events::<T>::update_system);
        }
        self
    }

    pub fn new(window_descriptor: WindowDescriptor) -> Self {
        let (window, event_loop) = create_window(window_descriptor);
        let mut world = World::default();
        let default_font_id = uuid::Uuid::new_v4();
        let renderer = pollster::block_on(Renderer::new(&window, default_font_id));
        world.insert_resource(UiHandler::new());
        world.insert_resource(renderer);
        let window_size = window.inner_size();
        world.send_event(WindowCreated {
            width: window_size.width as f32,
            height: window_size.height as f32,
        });

        let app = App {
            window,
            world,
            event_loop: Some(event_loop),
            schedule: Schedule::default(),
        };

        app.add_default_system_resources().add_assets()
    }

    pub fn spawn<T: Bundle>(mut self, component: T) -> Self {
        self.world.spawn(component);

        self
    }

    pub fn spawn_bundle<T: Bundle>(mut self, bundle: T) -> Self {
        self.world.spawn(bundle);

        self
    }

    pub fn add_system<Params>(mut self, system: impl IntoSystemDescriptor<Params>) -> Self {
        self.schedule.add_system_to_stage(CoreStage::Update, system);
        self
    }

    pub fn add_system_to_stage<Params>(
        mut self,
        system: impl IntoSystemDescriptor<Params>,
        stage: impl StageLabel,
    ) -> Self {
        self.schedule.add_system_to_stage(stage, system);
        self
    }

    pub fn run(self) {
        pollster::block_on(self.run_async());
    }

    pub fn update(&mut self) {
        self.schedule.run(&mut self.world);
    }

    pub async fn run_async(mut self) {
        env_logger::init();

        let event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| {
            let window = &self.window;
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested {} => *control_flow = ControlFlow::Exit,

                    WindowEvent::KeyboardInput { ref input, .. } => {
                        if let Some(key_code) = input.virtual_keycode {
                            self.world.send_event(KeyboardInput {
                                state: input.state,
                                key_code,
                            });
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        self.world.send_event(MouseButtonInput {
                            state: *state,
                            button: *button,
                        });
                    }
                    WindowEvent::Resized(physical_size) => {
                        let mut renderer = self.world.get_resource_mut::<Renderer>().unwrap();
                        renderer.resize(*physical_size);
                        self.world.send_event(WindowResized {
                            width: physical_size.width as f32,
                            height: physical_size.height as f32,
                        });
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        let mut renderer = self.world.get_resource_mut::<Renderer>().unwrap();
                        renderer.resize(**new_inner_size);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.world.send_event(CursorMoved {
                            position: *position,
                        });
                    }
                    _ => {}
                },

                Event::RedrawEventsCleared => *control_flow = ControlFlow::Poll,
                Event::MainEventsCleared => {
                    self.update();
                }

                _ => (),
            }
        });
    }
}

pub struct FrameCounterInfo {
    fps: i32,
    frame_count: i32,
    frame_time: i32,
}

pub fn diagnostic_system(time: Res<Time>) {
    let delta_seconds = time.raw_delta_seconds_f64();
    if delta_seconds == 0.0 {
        return;
    }

    dbg!(delta_seconds * 1000.0);

    dbg!(1.0 / delta_seconds);
}
