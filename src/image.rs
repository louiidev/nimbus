use std::{collections::HashMap, sync::Arc};

use bevy_ecs::system::Resource;
use glam::Vec2;
use wgpu::{Extent3d, Sampler, Texture, TextureDimension, TextureFormat, TextureView};

#[derive(Resource, Default)]
pub struct ImageBindGroups {
    pub values: HashMap<uuid::Uuid, Arc<wgpu::BindGroup>>,
}

pub const DEFAULT_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub texture_descriptor: wgpu::TextureDescriptor<'static>,
    pub texture_view_descriptor: Option<wgpu::TextureViewDescriptor<'static>>,
}

impl Default for Image {
    fn default() -> Self {
        let format = DEFAULT_TEXTURE_FORMAT;
        let data = vec![255; 1];
        Image {
            data,
            texture_descriptor: wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                format,
                dimension: wgpu::TextureDimension::D2,
                label: None,
                mip_level_count: 1,
                sample_count: 1,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            },
            texture_view_descriptor: None,
        }
    }
}

/// Used to calculate the volume of an item.
pub trait Volume {
    fn volume(&self) -> usize;
}

impl Volume for Extent3d {
    /// Calculates the volume of the [`Extent3d`].
    fn volume(&self) -> usize {
        (self.width * self.height * self.depth_or_array_layers) as usize
    }
}

impl Image {
    pub fn new_fill(
        size: Extent3d,
        dimension: TextureDimension,
        pixel: &[u8],
        format: TextureFormat,
    ) -> Self {
        let mut value = Image::default();
        value.texture_descriptor.format = format;
        value.texture_descriptor.dimension = dimension;
        value.resize(size);

        debug_assert!(
            pixel.len() <= value.data.len(),
            "Fill data must fit within pixel buffer."
        );

        for current_pixel in value.data.chunks_exact_mut(pixel.len()) {
            current_pixel.copy_from_slice(pixel);
        }
        value
    }

    pub fn resize(&mut self, size: Extent3d) {
        self.texture_descriptor.size = size;
        self.data.resize(size.volume(), 0);
    }
}

#[derive(Debug)]
pub struct GpuImage {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub texture_format: TextureFormat,
    pub size: Vec2,
}
