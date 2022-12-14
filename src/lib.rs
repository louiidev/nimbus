use std::collections::HashMap;

use bevy_ecs::{
    prelude::{Bundle, Events},
    schedule::{IntoSystemDescriptor, Schedule, Stage, StageLabel, SystemStage},
    system::Resource,
    world::{FromWorld, World},
};

use events::{WindowCreated, WindowResized};
use image::load_from_memory;
use renderer::{
    render_system,
    texture::{self, Texture},
    Renderer,
};

use resource_utils::Asset;
use transform::transform_propagate_system;
use window::{create_window, WindowDescriptor};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
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
pub mod resource_utils;
pub mod transform;
pub mod window;

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
}

pub struct App {
    world: World,
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
            .add_stage(CoreStage::Render, SystemStage::parallel());

        self.add_event::<WindowResized>()
            .add_event::<WindowCreated>();
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

        app.add_default_stages();

        app.world.insert_resource::<Asset<Texture>>(Asset::new());

        app.add_system_to_stage(render_system, CoreStage::Render)
            .add_system_to_stage(transform_propagate_system, CoreStage::PostUpdate)
            .add_system_to_stage(crate::camera::camera_system, CoreStage::PostUpdate)
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
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
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
