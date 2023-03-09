use bevy_ecs::{
    prelude::*,
    schedule::Schedule,
    system::{Res, Resource},
    world::{FromWorld, World},
};

use color::Color;
use ecs::{
    prelude::not,
    schedule::{IntoSystemConfig, SystemSet},
};
use editor::{Editor, EditorMode};
use events::{CursorMoved, KeyboardInput, MouseButtonInput, WindowCreated, WindowResized};
use font::FontData;
use font_atlas::FontAtlasSet;
use internal_image::{Image, DEFAULT_TEXTURE_FORMAT};
use math::{UVec2, Vec2};
use renderer::{render_system, texture::Texture, upload_images_to_gpu, Renderer};

use resources::{
    inputs::{input_system, InputController},
    utils::Assets,
};
use texture_atlas::TextureAtlas;
use time::Time;
use transform::transform_propagate_system;
use ui::UiHandler;
use wgpu::{Extent3d, TextureDimension};
use window::{create_window, Window, WindowDescriptor, WinitWindow};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub const DEFAULT_TEXTURE_ID: uuid::Uuid =
    uuid::Uuid::from_u128(0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8u128);

pub const DEFAULT_FONT_ID: uuid::Uuid =
    uuid::Uuid::from_u128(0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d9u128);

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

pub mod editor;
pub mod font;
mod font_atlas;
pub mod loaders;
pub mod ray;
pub mod utils;

pub use bevy_ecs as ecs;
pub use glam as math;
pub use winit;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreSet {
    /// The [`Stage`](bevy_ecs::schedule::Stage) that runs before all other app stages.
    First,
    /// The [`Stage`](bevy_ecs::schedule::Stage) that runs before [`CoreSet::Update`].
    PreUpdate,
    /// The [`Stage`](bevy_ecs::schedule::Stage) responsible for doing most app logic. Systems should be registered here by default.
    Update,
    /// The [`Stage`](bevy_ecs::schedule::Stage) that runs after [`CoreSet::Update`].
    PostUpdate,
    PrepareRenderer,
    Render,
    /// Cleanup render cache that runs after [`CoreSet::Render`].
    PostRender,
}

impl CoreSet {
    /// The sets defined in this enum are configured to run in order,
    pub fn base_schedule() -> Schedule {
        use CoreSet::*;
        let mut schedule = Schedule::new();

        // Create "stage-like" structure using buffer flushes + ordering
        schedule.configure_set(Update.after(PreUpdate).before(PostUpdate));
        schedule.configure_set(PrepareRenderer.after(PostUpdate).before(Render));
        schedule.configure_set(PostRender.after(Render));

        schedule
    }
}

// TODO: move this
#[derive(Resource, Default)]
pub struct ClearColor(pub Color);

pub struct App {
    pub world: World,
    schedule: Schedule,
    event_loop: Option<EventLoop<()>>,
    window: WinitWindow,
}

fn run_if_game_mode(editor: Res<Editor>) -> bool {
    editor.mode == EditorMode::Game
}

fn run_if_editor_mode(editor: Res<Editor>) -> bool {
    editor.mode == EditorMode::Editor
}

impl App {
    fn add_default_stages(&mut self) {
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
        self.init_resource::<FontAtlasSet>();
        self.add_asset::<FontData>();
        self.init_resource::<Editor>();
        self = self.insert_resource::<ClearColor>(ClearColor(Color::rgb_u8(52, 21, 174)));
        let image = Image::new_fill(
            Extent3d::default(),
            TextureDimension::D2,
            &[255u8; 4],
            DEFAULT_TEXTURE_FORMAT,
        );
        self.load_texture_with_id_image(image, DEFAULT_TEXTURE_ID);

        let font =
            FontData::try_from_bytes(include_bytes!("./default_assets/FiraSans-Bold.ttf")).unwrap();

        self.load_font_with_id(
            include_bytes!("./default_assets/FiraSans-Bold.ttf"),
            DEFAULT_FONT_ID,
        )
        .expect("Default font failed to load");

        let mut ui_handler = self.world.get_resource_mut::<UiHandler>().unwrap();

        ui_handler.fonts.insert(DEFAULT_FONT_ID, font);

        self
    }

    fn add_default_system_resources(mut self) -> Self {
        self.add_default_stages();
        self.init_resource::<InputController>();

        self.init_resource::<Time>();
        self.add_internal_system(render_system.in_set(CoreSet::Render))
            .add_internal_system(transform_propagate_system.in_set(CoreSet::PostUpdate))
            .add_internal_system(crate::camera::camera_system.in_set(CoreSet::PreUpdate))
            .add_internal_system(input_system.in_set(CoreSet::PreUpdate))
            .add_internal_system(
                upload_images_to_gpu
                    .in_set(CoreSet::PostUpdate)
                    .before(CoreSet::PrepareRenderer),
            )
            .init_2d_renderer()
            .init_ui_renderer()
    }

    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    pub fn insert_resource<R: Resource + FromWorld>(mut self, resource: R) -> Self {
        self.world.insert_resource::<R>(resource);

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
                .add_system(Events::<T>::update_system.in_set(CoreSet::First));
        }
        self
    }

    pub fn new(window_descriptor: WindowDescriptor) -> Self {
        let (winit_window, event_loop) = create_window(window_descriptor);

        let mut world = World::default();
        let default_font_id = uuid::Uuid::new_v4();
        let renderer = pollster::block_on(Renderer::new(&winit_window, default_font_id));

        world.insert_resource(renderer);
        let window_size = winit_window.inner_size();
        world.insert_resource(UiHandler::new(Vec2::new(
            window_size.width as f32,
            window_size.height as f32,
        )));

        let app = App {
            window: winit_window,
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

    pub fn add_internal_system<Params>(mut self, system: impl IntoSystemConfig<Params>) -> Self {
        self.schedule.add_system(system);

        self
    }

    pub fn add_system<Params>(mut self, system: impl IntoSystemConfig<Params>) -> Self {
        #[cfg(debug_assertions)]
        self.schedule
            .add_system(system.in_set(CoreSet::Update).run_if(not(run_if_game_mode)));

        #[cfg(not(debug_assertions))]
        self.schedule.add_system(system.in_set(CoreSet::Update));

        self
    }

    pub fn add_editor_system<Params>(mut self, system: impl IntoSystemConfig<Params>) -> Self {
        #[cfg(debug_assertions)]
        self.schedule.add_system(system.run_if(run_if_editor_mode));
        self
    }

    pub fn add_global_system<Params>(mut self, system: impl IntoSystemConfig<Params>) -> Self {
        #[cfg(debug_assertions)]
        self.schedule.add_system(system.in_set(CoreSet::Update));
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

        self.world.send_event(WindowCreated {
            width: self.window.inner_size().width as f32,
            height: self.window.inner_size().height as f32,
            scale: self.window.scale_factor() as f32,
        });

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
                            window_size: window.inner_size(),
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
