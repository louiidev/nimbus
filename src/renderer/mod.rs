use std::cell::RefCell;

use bevy_ecs::{
    schedule::{Stage, SystemStage},
    system::{Query, Res, ResMut, Resource},
    world::World,
};

use wgpu::{CommandEncoder, RenderPass};
use winit::window::Window;

use crate::{
    camera::Camera,
    resource_utils::{Asset, ResourceVec},
    transform::Transform,
    App, CoreStage,
};

use texture::Texture;

use self::{
    plugin_2d::{DefaultImageSampler, Renderer2D, SpritePipeline},
    sprite::Sprite,
    sprite_batching::{render_sprite_batches, SpriteBatch},
};

#[derive(Resource)]
pub struct Renderer {
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
}

mod mesh;
mod plugin_2d;
pub mod sprite;
pub mod sprite_batching;
pub mod texture;

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::DX12);
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
        }
    }

    pub fn load_texture_data(&self, bytes: &[u8]) -> Texture {
        println!("length: {}", bytes.len());
        let image = image::load_from_memory(bytes).unwrap();

        use image::GenericImageView;
        let dimensions = image.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image.to_rgb8(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture {
            texture,
            view,
            dimensions,
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
    sprite_batch: Res<ResourceVec<SpriteBatch>>,
    mut camera: Query<(&mut Camera)>,
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

    let bind_group = camera.bind_group.take().unwrap();
    {
        let camera_bind_group = bind_group.as_ref();

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
            &sprite_batch.value,
            &mut render_pass,
            &sprite_pipeline,
            camera_bind_group,
        );
    }

    renderer
        .queue
        .submit(std::iter::once(command_encoder.finish()));
    output.present();
}
