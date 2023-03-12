use bevy_ecs::{schedule::IntoSystemConfig, system::Resource};
use wgpu::{
    include_wgsl, BindGroupLayout, BlendState, FragmentState, FrontFace, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, VertexState,
};

use crate::{resources::utils::ResourceVec, App, CoreSet};

use super::{
    debug_drawing::{
        prepare_debug_mesh_to_render, DebugMesh, DebugMeshPipeline, PreparedDebugMeshItem,
    },
    mesh::{prepare_mesh_to_render, Mesh, MeshVertex, PreparedMeshItem},
    prepare_camera_buffers::prepare_camera_3d_bindgroup,
    Renderer,
};

#[derive(Resource)]
pub struct MeshPipeline {
    pub render_pipeline: RenderPipeline,
    pub texture_bind_group_layout: BindGroupLayout,
    pub camera_bind_group_layout: BindGroupLayout,
}

impl MeshPipeline {
    pub fn new(renderer: &Renderer) -> Self {
        let shader = renderer
            .device
            .create_shader_module(include_wgsl!("../shaders/mesh.wgsl"));

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
                    label: Some("Mesh Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let binding = [Some(wgpu::ColorTargetState {
            format: renderer.config.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let descriptor = RenderPipelineDescriptor {
            vertex: VertexState {
                entry_point: "vertex",
                buffers: &[MeshVertex::desc()],
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

impl App {
    pub fn initialize_mesh_rendering(mut self) -> Self {
        let renderer = self.world.get_resource::<Renderer>().unwrap();

        self.world.insert_resource(MeshPipeline::new(renderer));

        let renderer = self.world.get_resource::<Renderer>().unwrap();

        self.world.insert_resource(DebugMeshPipeline::new(renderer));

        let prepared_mesh_resource: Vec<PreparedMeshItem> = Vec::new();

        self.world
            .insert_resource(ResourceVec::new(prepared_mesh_resource));

        let prepared_debug_mesh_resource: Vec<PreparedDebugMeshItem> = Vec::new();

        self.world
            .insert_resource(ResourceVec::new(prepared_debug_mesh_resource));

        let debug_mesh_resource: Vec<DebugMesh> = Vec::new();

        self.world
            .insert_resource(ResourceVec::new(debug_mesh_resource));

        self.schedule
            .add_system(prepare_camera_3d_bindgroup.in_base_set(CoreSet::PrepareRenderer));

        self.schedule.add_system(
            prepare_mesh_to_render
                .in_base_set(CoreSet::PrepareRenderer)
                .after(prepare_camera_3d_bindgroup),
        );

        self.schedule.add_system(
            prepare_debug_mesh_to_render
                .in_base_set(CoreSet::PrepareRenderer)
                .after(prepare_camera_3d_bindgroup),
        );

        self
    }
}
