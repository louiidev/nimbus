use bevy_ecs::{
    prelude::{Bundle, Events},
    schedule::{IntoSystemDescriptor, Schedule, Stage, StageLabel, SystemStage},
    system::{Res, Resource},
    world::{FromWorld, World},
};

use events::{KeyboardInput, WindowCreated, WindowResized};
use renderer::{render_system, renderable::RenderCache, texture::Texture, Renderer};

use resources::{
    inputs::{input_system, Input},
    utils::Asset,
};
use time::Time;
use transform::transform_propagate_system;
use window::{create_window, WindowDescriptor};
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{self, ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

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
    /// The [`Stage`](bevy_ecs::schedule::Stage) that runs after all other app stages.
    Last,
    PrepareRenderer,
    Render,
    // Cleanup render cache
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
        let renderer = pollster::block_on(Renderer::new(&window));
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
            .add_stage(CoreStage::Last, SystemStage::parallel())
            .add_stage(CoreStage::PrepareRenderer, SystemStage::parallel())
            .add_stage(CoreStage::Render, SystemStage::parallel())
            .add_stage(CoreStage::PostRender, SystemStage::parallel());

        self.add_event::<WindowResized>()
            .add_event::<WindowCreated>()
            .add_event::<KeyboardInput>();
    }

    fn add_default_system_resources(mut self) -> Self {
        self.add_default_stages();

        self.world.insert_resource::<Asset<Texture>>(Asset::new());

        self.init_resource::<Input<VirtualKeyCode>>();

        self.init_resource::<Time>();

        self.init_resource::<RenderCache>();

        self.add_system_to_stage(render_system, CoreStage::Render)
            .add_system_to_stage(transform_propagate_system, CoreStage::PostUpdate)
            .add_system_to_stage(crate::camera::camera_system, CoreStage::PostUpdate)
            .add_system_to_stage(input_system, CoreStage::PreUpdate)
            .init_2d_renderer()
    }

    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
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
        let renderer = pollster::block_on(Renderer::new(&window));
        world.insert_resource(renderer);
        let window_size = window.inner_size();
        world.send_event(WindowCreated {
            width: window_size.width as f32,
            height: window_size.height as f32,
        });

        let mut app = App {
            window,
            world,
            event_loop: Some(event_loop),
            schedule: Schedule::default(),
        };

        app.add_default_system_resources()
    }

    pub fn load_texture(&mut self, bytes: &[u8]) -> uuid::Uuid {
        let renderer = self.world.get_resource::<Renderer>().unwrap();

        let texture = Texture::from_bytes(&renderer.device, &renderer.queue, bytes, "None");

        let texture_id = uuid::Uuid::new_v4();

        let mut textures = self.world.get_resource_mut::<Asset<Texture>>().unwrap();

        textures.data.insert(texture_id, texture);

        texture_id
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
                    _ => {}
                },

                Event::RedrawRequested(_) => {
                    self.update();
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    window.request_redraw();
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
