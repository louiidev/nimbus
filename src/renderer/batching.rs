use crate::{arena::ArenaId, mesh::Mesh, shader::Shader};
use wgpu::{util::DeviceExt, BindGroup, Buffer};

use super::Renderer;

#[derive(Debug)]
pub struct DrawCall {
    pub(crate) shader: ArenaId<Shader>,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) vert_len: u32,
    pub(crate) indices_len: u32,
    pub(crate) bind_groups: Vec<BindGroup>,
    pub(crate) index_format: wgpu::IndexFormat,
}

impl Renderer {
    pub(crate) fn create_draw_calls_from_meshes(&mut self) -> Vec<DrawCall> {
        let meshes = self.meshes.drain(0..).collect::<Vec<Mesh>>();

        let mut current_batch_texture_handle = ArenaId::default();
        let mut current_material_handle_id = ArenaId::default();
        let mut batches: Vec<Mesh> = Vec::new();

        for mesh in meshes {
            if current_batch_texture_handle == mesh.material.texture
                && current_material_handle_id == mesh.material.shader
            {
                let length = batches.len();

                let current_mesh = &mut batches[length - 1];
                let vert_count = current_mesh.vertices.len();
                let indices = mesh.indices.add(vert_count);

                current_mesh.concat(mesh.vertices, indices);
            } else {
                current_batch_texture_handle = mesh.material.texture;
                current_material_handle_id = mesh.material.shader;
                batches.push(mesh);
            }
        }

        batches
            .iter()
            .map(|mesh| {
                let index_format = mesh.indices.wgpu_index_format();
                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: &mesh
                                .vertices
                                .iter()
                                .map(|v| v.get_bytes())
                                .flatten()
                                .collect::<Vec<u8>>(),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: mesh.indices.cast_slice(),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                let shader = self
                    .shaders
                    .get(mesh.material.shader)
                    .expect("Cant find material for batch");

                let texture = self.textures.get(mesh.material.texture).unwrap();
                let sampler = self.samplers.get(texture.sampler).unwrap();

                let bind_groups = vec![texture.create_bind_group(
                    &self.device,
                    &shader.pipeline.get_bind_group_layout(1),
                    &sampler,
                )];

                DrawCall {
                    vertex_buffer,
                    index_buffer,
                    bind_groups,
                    vert_len: mesh.vertices.len() as _,
                    indices_len: mesh.indices.len() as _,
                    shader: mesh.material.shader,
                    index_format,
                }
            })
            .collect()
    }

    pub(crate) fn prepare_ui_mesh_batch(&mut self) -> Vec<DrawCall> {
        let mut meshes = self.ui_render_data.drain(0..).collect::<Vec<Mesh>>();

        // meshes.sort_by(|a, b| {
        //     a.sort_value
        //         .partial_cmp(&b.sort_value)
        //         .unwrap_or(std::cmp::Ordering::Equal)
        // });

        let mut current_batch_texture_handle = ArenaId::default();
        let mut current_material_handle_id = ArenaId::default();
        let mut batches: Vec<Mesh> = Vec::new();

        for mesh in meshes {
            if current_batch_texture_handle == mesh.material.texture
                && current_material_handle_id == mesh.material.shader
            {
                let length = batches.len();

                let current_mesh = &mut batches[length - 1];
                let vert_count = current_mesh.vertices.len();
                let indices = mesh.indices.add(vert_count);

                current_mesh.concat(mesh.vertices, indices);
            } else {
                current_batch_texture_handle = mesh.material.texture;
                current_material_handle_id = mesh.material.shader;
                batches.push(mesh);
            }
        }

        batches
            .iter()
            .map(|batch| {
                let index_format = batch.indices.wgpu_index_format();
                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: &batch
                                .vertices
                                .iter()
                                .map(|v| v.get_bytes())
                                .flatten()
                                .collect::<Vec<u8>>(),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: batch.indices.cast_slice(),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                let mut bind_groups = Vec::default();

                let shader = self
                    .shaders
                    .get(batch.material.shader)
                    .expect("Cant find material for batch");

                let texture = self.textures.get(batch.material.texture).unwrap();
                let sampler = self.samplers.get(texture.sampler).unwrap();
                let texture_bind_group = texture.create_bind_group(
                    &self.device,
                    &shader.pipeline.get_bind_group_layout(1),
                    &sampler,
                );

                bind_groups.push(texture_bind_group);

                DrawCall {
                    vertex_buffer,
                    index_buffer,
                    bind_groups,
                    vert_len: batch.vertices.len() as _,
                    indices_len: batch.indices.len() as _,
                    shader: batch.material.shader,
                    index_format,
                }
            })
            .collect()
    }
}
