use std::collections::{BTreeSet, HashMap};

use egui_wgpu_backend::RenderPass;
use glam::{Mat4, Vec3};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::{
    include_wgsl, util::DeviceExt, BindGroup, BindGroupLayout, BindingType, Buffer, CommandEncoder,
    PrimitiveTopology, Sampler, ShaderStages, SurfaceConfiguration, SurfaceTexture, TextureView,
};
use yakui_wgpu::{SurfaceInfo, YakuiWgpu};

use crate::{
    arena::{Arena, ArenaId},
    components::color::Color,
};

use self::{
    batching::DrawCall,
    bind_groups::BindGroupLayoutBuilder,
    camera::{Camera, CameraUniform, Projection},
    errors::RenderError,
    font_atlas::FontAtlas,
    fonts::{Font, FontSizeKey},
    mesh::{Mesh, MeshAttribute},
    shader::{PipelineBuilder, Shader},
    texture::{Texture, TextureSamplerType},
    ui::Layout,
};

pub mod batching;
pub mod bind_groups;
pub mod camera;
pub mod cube;
// pub mod depth_pass;
pub mod drawing;
mod dynamic_texture_atlas_builder;
pub mod errors;
mod font_atlas;
pub mod fonts;
pub mod line;
pub mod material;
pub mod mesh;
pub mod model;

pub mod quad;
pub mod rect;
pub mod shader;
// pub mod shapes;
pub mod sprite;
pub mod text;
pub mod texture;
pub mod texture_atlas;
pub mod transform;
pub mod ui;

pub struct ShaderMap {
    pub default: ArenaId<Shader>,
    pub line: ArenaId<Shader>,
    pub ui: ArenaId<Shader>,
}

pub struct RenderContext {
    pub(crate) output: SurfaceTexture,
    pub(crate) view: TextureView,
    pub(crate) command_encoder: CommandEncoder,
}

pub struct Renderer {
    pub(crate) fonts: Arena<Font>,
    pub(crate) font_atlases: HashMap<(FontSizeKey, ArenaId<Font>), FontAtlas>,
    pub textures: Arena<Texture>,
    pub device: wgpu::Device,
    pub(crate) meshes: Vec<Mesh>,
    pub(crate) default_texture_samplers: HashMap<TextureSamplerType, ArenaId<Sampler>>,
    pub samplers: Arena<Sampler>,
    camera_bind_group_layout: BindGroupLayout,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: SurfaceConfiguration,
    shaders: Arena<Shader>,
    pub(crate) default_shaders: ShaderMap,
    #[cfg(feature = "egui")]
    egui_render_pass: egui_wgpu_backend::RenderPass,
    pub(crate) ui_render_data: Vec<Mesh>,
    pub(crate) current_layout: Vec<Layout>,
    pub(crate) depth_texture_handle: ArenaId<Texture>,
    pub(crate) ui_projection: Mat4,
    yakui_wgpu: YakuiWgpu,
}

impl Renderer {
    /// Creates a  [`RenderBuddy`] Instance
    /// Creates a WGPU Surface, Instance, Device and Queue
    /// Requires async to request instance adapter
    pub async fn new<W>(window: &W, viewport_size: (u32, u32)) -> Result<Self, RenderError>
    where
        W: HasRawWindowHandle + HasRawDisplayHandle,
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window)? };

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Some(it) => it,
            None => return Err(RenderError::new("Unable to request adapter from wgpu")),
        };

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: viewport_size.0,
            height: viewport_size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::default(),
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // Webgl 2 for web until WGPU is fully supported
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await?;

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

        let depth_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        let mut samplers = Arena::new();

        let default_sampler_linear = samplers.insert(default_sampler_linear);
        let default_sampler_nearest = samplers.insert(default_sampler_nearest);
        let depth_texture_sampler_handle = samplers.insert(depth_texture_sampler);

        let camera_bind_group_layout = BindGroupLayoutBuilder::new()
            .append(
                ShaderStages::VERTEX,
                BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                None,
            )
            .build(&device, Some("camera_bind_group_layout"));

        let blank_texture = Texture::create_blank_texture(&device, &queue, default_sampler_nearest);

        let mut textures = Arena::new();

        let depth_texture = Texture::create_depth_texture(
            &device,
            &surface_config,
            depth_texture_sampler_handle.clone(),
        );

        textures.insert(blank_texture);
        let depth_texture_handle = textures.insert(depth_texture);

        let default_shader = PipelineBuilder::new(include_wgsl!("./default_shaders/default.wgsl"))
            .build(&device, &surface_config);

        let default_ui_shader =
            PipelineBuilder::new(include_wgsl!("./default_shaders/default.wgsl"))
                .with_depth(false)
                .build(&device, &surface_config);

        let line_shader = PipelineBuilder::new(include_wgsl!("./default_shaders/line.wgsl"))
            .with_topology(PrimitiveTopology::LineStrip)
            .with_vertex_attributes(BTreeSet::from([
                MeshAttribute::Position,
                MeshAttribute::Color,
            ]))
            .build(&device, &surface_config);

        let yakui_wgpu = YakuiWgpu::new(&device, &queue);

        let mut renderer = Self {
            #[cfg(feature = "egui")]
            egui_render_pass: RenderPass::new(&device, surface_format, 1),
            camera_bind_group_layout,
            font_atlases: HashMap::default(),
            fonts: Arena::new(),
            device,
            meshes: Vec::default(),
            textures,
            queue,
            surface,
            surface_config,
            samplers,
            default_texture_samplers: HashMap::from([
                (TextureSamplerType::Linear, default_sampler_linear),
                (TextureSamplerType::Nearest, default_sampler_nearest),
                (TextureSamplerType::Depth, depth_texture_sampler_handle),
            ]),
            shaders: Arena::new(),
            default_shaders: ShaderMap {
                default: ArenaId::first(),
                line: ArenaId::second(),
                ui: ArenaId::first(),
            },
            ui_render_data: Vec::default(),
            current_layout: Vec::default(),
            depth_texture_handle,
            ui_projection: Mat4::orthographic_rh(
                0.,
                viewport_size.0 as f32,
                viewport_size.1 as f32,
                0.,
                0.,
                1000.,
            ),
            yakui_wgpu,
        };

        renderer.default_shaders.default = renderer.shaders.insert(default_shader);
        renderer.default_shaders.line = renderer.shaders.insert(line_shader);
        renderer.default_shaders.ui = renderer.shaders.insert(default_ui_shader);

        renderer.fonts.insert(
            Font::try_from_bytes(include_bytes!("./default_font/Roboto-Regular.ttf")).unwrap(),
        );

        Ok(renderer)
    }
    /// Begin the render process by prepping the [`RenderContext`]
    pub fn begin(&self) -> RenderContext {
        let output = self
            .surface
            .get_current_texture()
            .expect("Missing current texture in surface");

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        RenderContext {
            output,
            view,
            command_encoder,
        }
    }

    pub fn create_shader(&mut self, pipeline_builder: PipelineBuilder) -> ArenaId<Shader> {
        self.shaders
            .insert(pipeline_builder.build(&self.device, &self.surface_config))
    }

    #[cfg(feature = "egui")]
    pub fn render_egui(
        &mut self,
        render_context: &mut RenderContext,
        textures: &egui::TexturesDelta,
        paint_jobs: &Vec<egui::ClippedPrimitive>,
    ) {
        self.egui_render_pass
            .add_textures(&self.device, &self.queue, textures)
            .unwrap();

        let size_in_pixels = self.get_viewport_size();
        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: size_in_pixels.0,
            physical_height: size_in_pixels.1,
            scale_factor: 1.,
        };

        self.egui_render_pass.update_buffers(
            &self.device,
            &self.queue,
            paint_jobs,
            &screen_descriptor,
        );

        self.egui_render_pass
            .execute(
                &mut render_context.command_encoder,
                &render_context.view,
                paint_jobs,
                &screen_descriptor,
                None,
            )
            .unwrap();
    }

    #[cfg(feature = "egui")]
    pub fn end_egui(&mut self, textures: egui::TexturesDelta) {
        self.egui_render_pass.remove_textures(textures).unwrap();
    }

    /// Render the queue
    /// takes in an optional clear color, potentially useful if you want to call render twice in one frame
    pub fn render(
        &mut self,
        render_context: &mut RenderContext,
        clear_color: Color,
        camera: &Camera,
        yakui_state: &mut yakui::Yakui,
    ) {
        let mesh_prepared_batch = self.create_draw_calls_from_meshes();
        let ui_mesh_prepared_batch = self.prepare_ui_mesh_batch();
        let camera_bind_group = camera.create_bind_group(
            &self.device,
            (self.surface_config.width, self.surface_config.height),
            &self.camera_bind_group_layout,
        );

        let ui_camera_bind_group = self.create_ui_camera_bind_group(&self.camera_bind_group_layout);

        {
            let mut render_pass =
                render_context
                    .command_encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &render_context.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(clear_color.into()),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.textures.get(self.depth_texture_handle).unwrap().view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });

            render_queued_draw_calls(
                &mesh_prepared_batch,
                &mut render_pass,
                &self.shaders,
                &camera_bind_group,
            );
        }

        self.yakui_wgpu.paint_with_encoder(
            yakui_state,
            &self.device,
            &self.queue,
            &mut render_context.command_encoder,
            SurfaceInfo {
                format: self.surface_config.format,
                color_attachment: &render_context.view,
                resolve_target: None,
                sample_count: 1,
            },
        )

        // {
        //     let mut render_pass =
        //         render_context
        //             .command_encoder
        //             .begin_render_pass(&wgpu::RenderPassDescriptor {
        //                 label: Some("Render Pass"),
        //                 color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //                     view: &render_context.view,
        //                     resolve_target: None,
        //                     ops: wgpu::Operations {
        //                         load: wgpu::LoadOp::Load,
        //                         store: true,
        //                     },
        //                 })],
        //                 depth_stencil_attachment: None,
        //             });

        //     render_queued_draw_calls(
        //         &ui_mesh_prepared_batch,
        //         &mut render_pass,
        //         &self.shaders,
        //         &ui_camera_bind_group,
        //     )
        // }
    }

    /// Presents the frame to WGPU for rendering
    /// Drops the [`RenderContext`]
    pub fn end_frame(&mut self, render_context: RenderContext) {
        self.queue
            .submit(std::iter::once(render_context.command_encoder.finish()));
        render_context.output.present();
    }

    /// Should be called when the window has been resized
    pub fn resize(&mut self, new_surface_size: (u32, u32)) {
        self.surface_config.width = new_surface_size.0;
        self.surface_config.height = new_surface_size.1;
        self.surface.configure(&self.device, &self.surface_config);

        self.replace_texture(
            self.depth_texture_handle,
            Texture::create_depth_texture(
                &self.device,
                &self.surface_config,
                *self
                    .default_texture_samplers
                    .get(&TextureSamplerType::Depth)
                    .unwrap(),
            ),
        );
    }

    pub fn append(&mut self, mut meshes: Vec<Mesh>) {
        self.meshes.append(&mut meshes)
    }

    pub fn push(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    pub fn push_ui(&mut self, mesh: Mesh) {
        self.ui_render_data.push(mesh);
    }

    pub(crate) fn get_viewport_size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }

    pub(crate) fn create_ui_camera_bind_group(
        &self,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        let camera_uniform = CameraUniform {
            view_proj: self.ui_projection.to_cols_array_2d(),
        };

        let camera_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                contents: bytemuck::cast_slice(&[camera_uniform]),
            });

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Camera bind group"),
            layout: &bind_group_layout,
        })
    }
}

fn render_queued_draw_calls<'a>(
    draw_calls: &'a Vec<DrawCall>,
    render_pass: &mut wgpu::RenderPass<'a>,
    shaders: &'a Arena<Shader>,
    camera_bind_group: &'a BindGroup,
) {
    let last_shader = ArenaId::default();

    for draw_call in draw_calls {
        if draw_call.shader != last_shader {
            let shader: &Shader = shaders
                .get(draw_call.shader)
                .expect("Mesh was given invalid pipeline id");
            render_pass.set_pipeline(&shader.pipeline);

            render_pass.set_bind_group(0, &camera_bind_group, &[]); // can probably do this once before the loop
        }

        for (i, bind_group) in draw_call.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32 + 1, &bind_group, &[]);
        }

        render_pass.set_vertex_buffer(0, draw_call.vertex_buffer.slice(..));
        if draw_call.indices_len > 0 {
            render_pass.set_index_buffer(draw_call.index_buffer.slice(..), draw_call.index_format);
            render_pass.draw_indexed(0..draw_call.indices_len, 0, 0..1);
        } else {
            render_pass.draw(0..draw_call.vert_len, 0..1);
        }
    }
}
