use bevy_ecs::{
    prelude::Component,
    system::{Query, Res, ResMut, Resource},
};

use glam::Vec2;
use uuid::Uuid;
use wgpu::{BindGroup, Buffer};
use winit::window::Window;

use crate::{camera::Camera, resources::utils::ResourceVec, time::Time};

use self::{plugin_2d::SpritePipeline, sprite_batching::render_sprite_batches};

pub const QUAD_INDICES: [u16; 6] = [0, 2, 3, 0, 1, 2];

pub const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
    Vec2::new(-0.5, -0.5),
    Vec2::new(0.5, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

pub const QUAD_UVS: [Vec2; 4] = [
    Vec2::new(0., 1.),
    Vec2::new(1., 1.),
    Vec2::new(1., 0.),
    Vec2::new(0., 0.),
];

#[derive(Resource)]
pub struct Renderer {
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub default_font_id: Uuid,
}

pub(crate) mod mesh;
pub(crate) mod plugin_2d;
pub(crate) mod renderable;
pub(crate) mod sprite_batching;
pub mod texture;
pub(crate) mod ui;

impl Renderer {
    pub async fn new(window: &Window, default_font_id: uuid::Uuid) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Couldn't find adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .expect("Error requesting device");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        Renderer {
            surface,
            device,
            queue,
            config,
            size,
            default_font_id,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        } else {
            panic!("Invalid size???");
        }
    }
}

pub fn render_system(
    renderer: Res<Renderer>,
    sprite_pipeline: Res<SpritePipeline>,
    sprite_batch: Res<ResourceVec<RenderBatchItem>>,
    mut camera: Query<&mut Camera>,
    mut time: ResMut<Time>,
) {
    let mut camera = camera.get_single_mut().unwrap();

    let output = renderer
        .surface
        .get_current_texture()
        .expect("Missing current texture in surface");

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut command_encoder =
        renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

    {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        render_sprite_batches(
            &sprite_batch.values,
            &mut render_pass,
            &sprite_pipeline,
            &camera.bind_groups,
        );

        // render_ui(&mut render_pass);
    }

    renderer
        .queue
        .submit(std::iter::once(command_encoder.finish()));
    output.present();

    time.update()
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Component)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 5]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Resource, Debug)]
pub struct RenderBatchItem {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    texture_bind_group: BindGroup,
    indices_len: u32,
    camera_bind_group_id: u8,
}

pub struct RenderBatchMeta<V> {
    pub(crate) texture_id: uuid::Uuid,
    pub(crate) vertices: Vec<V>,
    pub(crate) indices: Vec<u16>,
}

impl<V> RenderBatchMeta<V> {
    pub fn new(texture_id: uuid::Uuid, vertices: Vec<V>, indices: Vec<u16>) -> Self {
        Self {
            texture_id,
            vertices,
            indices,
        }
    }

    pub fn update(&mut self, mut vertices: Vec<V>, mut indices: Vec<u16>) {
        self.vertices.append(&mut vertices);
        self.indices.append(&mut indices);
    }
}
