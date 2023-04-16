pub mod debug_mesh;
pub mod drawing;
pub mod font_renderer;
pub mod mesh2d;
pub(crate) mod pipelines;
pub mod texture;
pub mod ui;

#[cfg(feature = "debug-egui")]
use egui::TextureId;
#[cfg(feature = "debug-egui")]
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
#[cfg(feature = "debug-egui")]
use egui_winit_platform::Platform;
use glam::{UVec2, Vec2, Vec3};
use sdl2::video::Window;
use std::{collections::HashMap, sync::Arc};
use wgpu::{Sampler, SurfaceConfiguration};

use crate::{
    arena::{Arena, ArenaId},
    camera::Camera,
    components::color::Color,
    internal_image::InternalImage,
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

pub struct Viewport {
    size: UVec2,
    scale: f32,
}

pub struct Renderer {
    // The GPU textures
    pub(crate) textures: Arena<Texture>,
    #[cfg(feature = "debug-egui")]
    pub egui_texture_map: HashMap<ArenaId, TextureId>,

    pub(crate) font_renderer: FontRenderer,

    // WGPU
    pub(crate) device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: SurfaceConfiguration,
    pub(crate) texture_samplers: HashMap<TextureSampler, Arc<Sampler>>,

    pub clear_color: Color,

    pub(crate) viewport: Viewport,

    // Render pipelines for each drawing type.
    // TODO: needs to support custom pipelines
    pub(crate) render_pipelines: HashMap<PipelineType, Pipeline>,
    // The batches, populated by the engine caller

    // Before rendering, we prep these meshes for render batching and drawing
    // mesh used for sprite and text drawing or anything 2d
    pub(crate) meshes2d: Vec<(Mesh2d, Vec3)>, // storing the transform translation for sorting
    // pub(crate) render_mesh_batch: Vec<PreparedMeshItem>,
    // Used for drawing lines and debug shapes
    // TODO: remove from release build
    pub(crate) debug_meshes: Vec<DebugMesh>,
    #[cfg(feature = "debug-egui")]
    egui_render_pass: RenderPass,
}
impl Renderer {
    pub fn get_viewport(&self) -> Vec2 {
        self.viewport.size.as_vec2()
    }

    pub async fn new(window: &Window, viewport: UVec2) -> Self {
        let size = window.size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&window).unwrap() };
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
            width: size.0,
            height: size.1,
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

        #[cfg(feature = "debug-egui")]
        let egui_render_pass = RenderPass::new(&device, surface_format, 1);

        let mut renderer = Self {
            textures: Arena::new(),
            #[cfg(feature = "debug-egui")]
            egui_texture_map: HashMap::default(),
            surface,
            device,
            queue,
            clear_color: Color::hex("#6b6ab3").unwrap(),
            render_pipelines: HashMap::default(),
            meshes2d: Vec::default(),
            // render_mesh_batch: Vec::default(),
            debug_meshes: Vec::default(),
            texture_samplers: HashMap::from([
                (TextureSampler::Linear, Arc::new(default_sampler_linear)),
                (TextureSampler::Nearest, Arc::new(default_sampler_nearest)),
            ]),
            viewport: Viewport {
                size: viewport,
                scale: 1.,
            },
            surface_config,
            font_renderer: FontRenderer::new(),
            #[cfg(feature = "debug-egui")]
            egui_render_pass,
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
            self.viewport.size = UVec2::new(new_size.x, new_size.y);
            self.surface_config.width = new_size.x;
            self.surface_config.height = new_size.y;
            self.surface.configure(&self.device, &self.surface_config);
        } else {
            panic!("Invalid size???");
        }
    }

    pub fn load_texture(&mut self, image: InternalImage) -> ArenaId {
        let texture =
            Texture::from_detailed_bytes(&self.device, &self.queue, &image.data, image.size);
        let filter = if let TextureSampler::Linear = texture.sampler {
            wgpu::FilterMode::Linear
        } else {
            wgpu::FilterMode::Nearest
        };
        #[cfg(feature = "debug-egui")]
        let id = self.egui_render_pass.egui_texture_from_wgpu_texture(
            &self.device,
            &texture.view,
            filter,
        );

        let arena_id = self.textures.insert(texture);

        #[cfg(feature = "debug-egui")]
        self.egui_texture_map.insert(arena_id, id);

        arena_id
    }

    pub fn replace_texture(&mut self, id: ArenaId, image: InternalImage) {
        let texture =
            Texture::from_detailed_bytes(&self.device, &self.queue, &image.data, image.size);
        *self.textures.get_mut(id).unwrap() = texture;
    }

    pub fn load_font(&mut self, bytes: &[u8]) -> ArenaId {
        let id = self.font_renderer.load_font(bytes).unwrap();

        id
    }

    pub(crate) fn get_texture_sampler(&self, sampler_type: TextureSampler) -> &Arc<Sampler> {
        self.texture_samplers.get(&sampler_type).unwrap()
    }

    pub fn render(
        &mut self,
        camera: &Camera,
        ui: &mut Ui,
        #[cfg(feature = "debug-egui")] egui_platform: &mut Platform,
        window: &Window,
    ) {
        let output = self
            .surface
            .get_current_texture()
            .expect("Missing current texture in surface");

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        #[cfg(feature = "debug-egui")]
        let full_output = egui_platform.end_frame(Some(window));
        #[cfg(feature = "debug-egui")]
        let paint_jobs = egui_platform.context().tessellate(full_output.shapes);

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

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

        #[cfg(feature = "debug-egui")]
        {
            let screen_descriptor = ScreenDescriptor {
                physical_width: self.surface_config.width,
                physical_height: self.surface_config.height,
                scale_factor: self.viewport.scale,
            };

            let tdelta: egui::TexturesDelta = full_output.textures_delta;
            self.egui_render_pass
                .add_textures(&self.device, &self.queue, &tdelta)
                .expect("add texture ok");
            self.egui_render_pass.update_buffers(
                &self.device,
                &self.queue,
                &paint_jobs,
                &screen_descriptor,
            );

            // Record all render passes.
            self.egui_render_pass
                .execute(
                    &mut command_encoder,
                    &view,
                    &paint_jobs,
                    &screen_descriptor,
                    None,
                )
                .unwrap();

            self.queue.submit(std::iter::once(command_encoder.finish()));
            output.present();

            self.egui_render_pass
                .remove_textures(tdelta)
                .expect("remove texture ok");
        }

        #[cfg(not(feature = "debug-egui"))]
        {
            self.queue.submit(std::iter::once(command_encoder.finish()));
            output.present();
        }
    }
}
