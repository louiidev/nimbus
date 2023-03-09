use std::sync::Arc;

use bevy_ecs::{schedule::IntoSystemConfig, system::Resource};
use wgpu::{
    include_wgsl, BindGroupLayout, BlendState, FragmentState, FrontFace, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, Sampler,
    VertexState,
};

use crate::{
    camera::CameraBundle, components::collider::debug_collider_picker,
    internal_image::ImageBindGroups, resources::utils::ResourceVec, window::Window, App, CoreSet,
};

use super::{
    shapes::prepare_shapes_for_batching, sprite_batching::prepare_sprites_for_batching,
    text::prepare_text_for_batching, RenderBatchItem, Renderer, Vertex,
};

#[derive(Resource)]
pub struct Renderer2D;

#[derive(Resource, Clone)]
pub struct DefaultImageSampler(pub(crate) Arc<Sampler>);

#[derive(Resource)]
pub struct SpriteRenderPipeline(RenderPipeline);

impl App {
    pub fn init_2d_renderer(mut self) -> Self {
        self.world.insert_resource(ImageBindGroups::default());

        self.world.insert_resource(Renderer2D);

        let renderer = self.world.get_resource::<Renderer>().unwrap();

        let default_sampler = {
            renderer.device.create_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            })
        };
        self.world.insert_resource(SpritePipeline::new(renderer));

        self.world
            .insert_resource(DefaultImageSampler(Arc::new(default_sampler)));
        self.schedule
            .add_system(prepare_sprites_for_batching.in_set(CoreSet::PrepareRenderer));

        self.schedule
            .add_system(prepare_shapes_for_batching.in_set(CoreSet::PrepareRenderer));

        self.schedule
            .add_system(prepare_text_for_batching.in_set(CoreSet::PrepareRenderer));

        self = self.add_editor_system(debug_collider_picker);

        let sprite_batch_resource: Vec<RenderBatchItem> = Vec::new();

        self.world
            .insert_resource(ResourceVec::new(sprite_batch_resource));

        self
    }
}

#[derive(Resource)]
pub struct SpritePipeline {
    pub render_pipeline: RenderPipeline,
    pub texture_bind_group_layout: BindGroupLayout,
    pub camera_bind_group_layout: BindGroupLayout,
}

impl SpritePipeline {
    pub fn new(renderer: &Renderer) -> Self {
        let shader = renderer
            .device
            .create_shader_module(include_wgsl!("../shaders/sprite_shader.wgsl"));

        let texture_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let camera_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });

        let render_pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let binding = [Some(wgpu::ColorTargetState {
            // 4.
            format: renderer.config.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let descriptor = RenderPipelineDescriptor {
            vertex: VertexState {
                entry_point: "vertex",
                buffers: &[Vertex::desc()],
                module: &shader,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fragment",
                targets: &binding,
            }),
            layout: Some(&render_pipeline_layout),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            label: Some("sprite_pipeline"),
            multiview: None,
        };

        Self {
            render_pipeline: renderer.device.create_render_pipeline(&descriptor),
            texture_bind_group_layout,
            camera_bind_group_layout,
        }
    }
}
