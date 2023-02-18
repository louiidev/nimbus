use std::{collections::HashMap, sync::Arc};

use bevy_ecs::system::Resource;
use glam::Vec2;
use wgpu::{Extent3d, Texture, TextureDimension, TextureFormat, TextureView};

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
    pub(crate) dirty: bool,
}

impl Default for Image {
    fn default() -> Self {
        let format = DEFAULT_TEXTURE_FORMAT;
        let data = vec![255; format.describe().block_size as usize];

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
            dirty: true,
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
    pub fn new(
        size: Extent3d,
        dimension: TextureDimension,
        data: Vec<u8>,
        format: TextureFormat,
    ) -> Self {
        let mut image = Self {
            data,
            ..Default::default()
        };
        image.texture_descriptor.dimension = dimension;
        image.texture_descriptor.size = size;
        image.texture_descriptor.format = format;
        image
    }

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

        debug_assert_eq!(
            pixel.len() % format.describe().block_size as usize,
            0,
            "Must not have incomplete pixel data."
        );

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
        self.data.resize(
            size.volume() * self.texture_descriptor.format.describe().block_size as usize,
            0,
        );
    }

    pub fn from_bytes(buffer: &[u8]) -> Image {
        let image = image::load_from_memory(buffer).unwrap();
        let rgba = image.to_rgba8();
        let size = Extent3d {
            width: rgba.width(),
            height: rgba.height(),
            depth_or_array_layers: 1,
        };
        Image::new(
            size,
            TextureDimension::D2,
            rgba.into_raw(),
            DEFAULT_TEXTURE_FORMAT,
        )
    }
}

#[test]
fn white_image() {
    let image = Image::new_fill(
        Extent3d::default(),
        TextureDimension::D2,
        &[255u8; 4],
        DEFAULT_TEXTURE_FORMAT,
    );
}
