use std::num::NonZeroU32;

use glam::UVec2;
use image::DynamicImage;
use wgpu::Extent3d;

#[derive(Default, PartialEq, Hash, Eq, Clone, Copy)]
pub enum TextureSampler {
    Linear,
    #[default]
    Nearest,
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub dimensions: UVec2,
    pub(crate) sampler: TextureSampler,
}

impl Texture {
    pub fn blank(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let size = Extent3d::default();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &[255u8; 4],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * size.width),
                rows_per_image: NonZeroU32::new(size.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            dimensions: UVec2::new(size.width, size.height),
            sampler: TextureSampler::default(),
        }
    }

    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) -> Self {
        let image = image::load_from_memory(bytes).unwrap();

        Self::from_image(device, queue, &image)
    }

    pub fn from_detailed_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        size: (usize, usize),
    ) -> Self {
        let size = Extent3d {
            width: size.0 as _,
            height: size.1 as _,
            depth_or_array_layers: 1,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let format_described = texture_descriptor.format.describe();
        let texture = device.create_texture(&texture_descriptor);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(format_described.block_size as u32 * size.width),
                rows_per_image: NonZeroU32::new(size.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            dimensions: UVec2::new(size.width, size.height),
            sampler: TextureSampler::default(),
        }
    }

    pub fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, image: &DynamicImage) -> Self {
        let rgba = image.to_rgba8();

        Texture::from_detailed_bytes(
            device,
            queue,
            image.as_bytes(),
            (rgba.width() as _, rgba.height() as _),
        )
    }
}
