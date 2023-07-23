use std::fmt::Debug;

use wgpu::{
    BindGroupLayout, BindingType, BlendState, FragmentState, FrontFace, PolygonMode,
    PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat,
    VertexBufferLayout, VertexState, VertexStepMode,
};

use crate::{bind_groups::BindGroupLayoutBuilder, material::Material, mesh::get_attribute_layout};

use super::Renderer;

pub struct Pipeline {
    pub(crate) render_pipeline: RenderPipeline,
    pub(crate) material: Box<dyn Material>,
}

impl Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("render_pipeline", &self.render_pipeline)
            .field("material", &self.material)
            .finish()
    }
}

impl Renderer {
    pub(crate) fn create_pipeline_from_material(
        &mut self,
        material: &impl Material,
    ) -> RenderPipeline {
        let shader = self.device.create_shader_module(material.shader());

        let vertex_attributes = material.vertex_attributes();

        let (vertex_attribute, offset) = get_attribute_layout(vertex_attributes.iter());

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: offset as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: vertex_attribute.as_slice(),
        };

        let bind_group_layouts: Vec<BindGroupLayout> =
            material.get_bind_group_layouts(&self.device);

        let texture_bind_group_layout = BindGroupLayoutBuilder::new()
            .append(
                ShaderStages::FRAGMENT,
                BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float {
                        filterable: material.filterable_texture(),
                    },
                },
                None,
            )
            .append(
                ShaderStages::FRAGMENT,
                BindingType::Sampler(if material.filterable_texture() {
                    wgpu::SamplerBindingType::Filtering
                } else {
                    wgpu::SamplerBindingType::NonFiltering
                }),
                None,
            )
            .build(&self.device, None);

        let mut predefined_bind_group_layouts = if material.has_texture() {
            vec![&self.camera_bind_group_layout, &texture_bind_group_layout]
        } else {
            vec![&self.camera_bind_group_layout]
        };
        predefined_bind_group_layouts.append(&mut bind_group_layouts.iter().map(|x| &*x).collect());

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Material Pipeline Layout"),
                    bind_group_layouts: predefined_bind_group_layouts.as_slice(),
                    push_constant_ranges: &[],
                });

        let binding = [Some(wgpu::ColorTargetState {
            format: self.surface_config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let descriptor = RenderPipelineDescriptor {
            vertex: VertexState {
                entry_point: "vertex",
                buffers: &[vertex_buffer_layout],
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
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: material.topology(),
                strip_index_format: None,
            },
            depth_stencil: if material.use_depth_stencil() {
                dbg!("FIRES");
                Some(wgpu::DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                })
            } else {
                None
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some(material.label()),
            multiview: None,
        };

        self.device.create_render_pipeline(&descriptor)
    }
}
