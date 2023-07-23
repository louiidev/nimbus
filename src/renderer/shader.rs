use std::collections::BTreeSet;

use wgpu::{
    BindingType, Device, FragmentState, FrontFace, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderStages, Surface,
    SurfaceConfiguration, TextureFormat, VertexBufferLayout, VertexState, VertexStepMode,
};

use crate::{
    bind_groups::BindGroupLayoutBuilder,
    mesh::{get_attribute_layout, MeshAttribute},
};

#[derive(Debug)]
pub struct Shader {
    pipeline: RenderPipeline,
}

pub struct PipelineBuilder<'a> {
    module: ShaderModuleDescriptor<'a>,
    topology: PrimitiveTopology,
    vertex_attributes: BTreeSet<MeshAttribute>,
    label: String,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(module: ShaderModuleDescriptor<'a>) -> Self {
        PipelineBuilder {
            module,
            topology: PrimitiveTopology::TriangleList,
            vertex_attributes: BTreeSet::from([
                MeshAttribute::Position,
                MeshAttribute::UV,
                MeshAttribute::Color,
            ]),
            label: "Generic Render Pipeline".to_string(),
        }
    }

    pub fn build(self, device: &Device, surface_config: &SurfaceConfiguration) -> Shader {
        let shader_module = device.create_shader_module(self.module);

        let vertex_attributes = self.vertex_attributes;

        let (vertex_attribute, offset) = get_attribute_layout(vertex_attributes.iter());

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: offset as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: vertex_attribute.as_slice(),
        };

        let texture_bind_group_layout = BindGroupLayoutBuilder::new()
            .append(
                ShaderStages::FRAGMENT,
                BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                None,
            )
            .append(
                ShaderStages::FRAGMENT,
                BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                None,
            )
            .build(&device, None);

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

        let bind_group_layouts = vec![&camera_bind_group_layout, &texture_bind_group_layout];

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: bind_group_layouts.as_slice(),
                push_constant_ranges: &[],
            });

        let binding = [Some(wgpu::ColorTargetState {
            format: surface_config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let descriptor = RenderPipelineDescriptor {
            vertex: VertexState {
                entry_point: "vertex",
                buffers: &[vertex_buffer_layout],
                module: &shader_module,
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fragment",
                targets: &binding,
            }),
            layout: Some(&render_pipeline_layout),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: self.topology,
                strip_index_format: None,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some(&self.label),
            multiview: None,
        };

        Shader {
            pipeline: device.create_render_pipeline(&descriptor),
        }
    }
}

impl Shader {}
