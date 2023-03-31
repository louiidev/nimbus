pub mod debug_mesh;
pub mod drawing;
pub mod font_renderer;
pub mod mesh2d;
pub(crate) mod pipelines;
pub mod texture;
pub mod ui;

use std::{collections::HashMap, sync::Arc};

use glam::{UVec2, Vec2, Vec3};
use wgpu::{Sampler, SurfaceConfiguration};
use winit::window::Window;

use crate::{
    arena::{Arena, ArenaId},
    camera::Camera,
    components::color::Color,
    systems::{
        prepare_render::{
            prepare_debug_mesh_for_batching, prepare_mesh2d_for_batching, prepare_ui_for_batching,
        },
        rendering::{render_2d_batch, render_debug_meshes, render_ui_batch},
    },
};

use self::{
    debug_mesh::{setup_debug_mesh_pipeline, DebugMesh},
    font_renderer::FontRenderer,
    mesh2d::{setup_mesh2d_pipeline, Mesh2d},
    pipelines::{Pipeline, PipelineType},
    texture::{Texture, TextureSampler},
    ui::Ui,
};

pub struct Renderer {
    // The GPU textures
    pub(crate) textures: Arena<Texture>,

    pub(crate) font_renderer: FontRenderer,

    // WGPU
    pub(crate) device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: SurfaceConfiguration,
    pub(crate) texture_samplers: HashMap<TextureSampler, Arc<Sampler>>,

    pub clear_color: Color,

    pub(crate) viewport: UVec2,

    // Render pipelines for each drawing type.
    // TODO: needs to support custom pipelines
    pub(crate) render_pipelines: HashMap<PipelineType, Pipeline>,
    // The batches, populated by the engine caller

    // Before rendering, we prep these meshes for render batching and drawing
    // mesh used for sprite and text drawing or anything 2d
    pub(crate) meshes2d: Vec<(Mesh2d, Vec3)>, // storing the transform translation for sorting
    pub(crate) ui_meshes: Vec<Mesh2d>,
    // pub(crate) render_mesh_batch: Vec<PreparedMeshItem>,
    // Used for drawing lines and debug shapes
    // TODO: remove from release build
    pub(crate) debug_meshes: Vec<DebugMesh>,
}
impl Renderer {
    pub fn get_viewport(&self) -> Vec2 {
        self.viewport.as_vec2()
    }

    pub async fn new(window: &Window, viewport: UVec2) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.describe().srgb)
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::default(),
        };

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
            .unwrap();

        surface.configure(&device, &surface_config);

        let default_sampler_nearest = {
            device.create_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            })
        };

        let default_sampler_linear = {
            device.create_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })
        };

        let mut renderer = Self {
            textures: Arena::new(),
            surface,
            device,
            queue,
            clear_color: Color::hex("#6b6ab3").unwrap(),
            render_pipelines: HashMap::default(),
            meshes2d: Vec::default(),
            ui_meshes: Vec::default(),
            // render_mesh_batch: Vec::default(),
            debug_meshes: Vec::default(),
            texture_samplers: HashMap::from([
                (TextureSampler::Linear, Arc::new(default_sampler_linear)),
                (TextureSampler::Nearest, Arc::new(default_sampler_nearest)),
            ]),
            viewport,
            surface_config,
            font_renderer: FontRenderer::new(),
        };

        renderer
            .render_pipelines
            .insert(PipelineType::Mesh2d, setup_mesh2d_pipeline(&renderer));

        renderer.render_pipelines.insert(
            PipelineType::DebugMesh,
            setup_debug_mesh_pipeline(&renderer),
        );

        let id = renderer
            .textures
            .insert(Texture::blank(&renderer.device, &renderer.queue));
        debug_assert!(id == ArenaId::first(), "Arena id out of sync");
        renderer
    }

    pub(crate) fn resize(&mut self, new_size: UVec2) {
        if new_size.x > 0 && new_size.y > 0 {
            self.viewport = UVec2::new(new_size.x, new_size.y);
            self.surface_config.width = new_size.x;
            self.surface_config.height = new_size.y;
            self.surface.configure(&self.device, &self.surface_config);
        } else {
            panic!("Invalid size???");
        }
    }

    pub fn load_texture(&mut self, bytes: &[u8]) -> ArenaId {
        let texture = Texture::from_bytes(&self.device, &self.queue, bytes);
        self.textures.insert(texture)
    }

    pub fn load_font(&mut self, bytes: &[u8]) -> ArenaId {
        let id = self.font_renderer.load_font(bytes).unwrap();

        id
    }

    pub(crate) fn get_texture_sampler(&self, sampler_type: TextureSampler) -> &Arc<Sampler> {
        self.texture_samplers.get(&sampler_type).unwrap()
    }

    // pub fn move_temp_image_batches_from_font_renderer(
    //     textures: &mut Arena<Texture>,
    //     font_renderer: &mut FontRenderer,
    //     device: &Device,
    //     queue: &Queue,
    // ) {
    //     let temp_images: HashMap<(FloatOrd, ArenaId), TempImageData> =
    //         font_renderer.images_to_upload.drain().collect();

    //     for (key, image) in temp_images {
    //         if let Some(key) = font_renderer.texture_mapping.get(&key).copied() {
    //             let texture = Texture::from_bytes(&device, &queue, &image.data);
    //             *textures.get_mut(key).unwrap() = texture;
    //         }
    //     }
    // }

    pub fn render(&mut self, camera: &Camera, ui: &mut Ui) {
        let output = self
            .surface
            .get_current_texture()
            .expect("Missing current texture in surface");

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // // TODO: think of a way to only have a single font cache
        // Renderer::move_temp_image_batches_from_font_renderer(
        //     &mut self.textures,
        //     &mut self.font_renderer,
        //     &self.device,
        //     &self.queue,
        // );

        // Renderer::move_temp_image_batches_from_font_renderer(
        //     &mut self.textures,
        //     &mut ui.font_renderer,
        //     &self.device,
        //     &self.queue,
        // );

        let sprite_batch = prepare_mesh2d_for_batching(self);
        let debug_mesh_batch = prepare_debug_mesh_for_batching(self);
        let ui_mesh_batch = prepare_ui_for_batching(ui, self);
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_2d_batch(
                &sprite_batch,
                &mut render_pass,
                &self.render_pipelines,
                &camera.bind_groups,
            );

            // render_mesh_batches(
            //     &self.render_mesh_batch,
            //     &mut render_pass,
            //     &mesh_pipeline,
            //     &camera.bind_groups,
            // );

            render_debug_meshes(
                &debug_mesh_batch,
                &mut render_pass,
                &self.render_pipelines,
                &camera.bind_groups,
            );

            render_ui_batch(
                &ui_mesh_batch,
                &mut render_pass,
                &self.render_pipelines,
                &camera.bind_groups,
            );
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        output.present();
    }
}
