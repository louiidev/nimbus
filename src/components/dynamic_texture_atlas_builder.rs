use super::{rect::Rect, texture_atlas::TextureAtlas};
use glam::{IVec2, Vec2};
use guillotiere::{size2, Allocation, AtlasAllocator};
use wgpu::TextureFormat;

#[derive(Clone)]
pub(crate) struct TempImageData {
    pub(crate) data: Vec<u8>,
    pub(crate) size: IVec2,
    pub(crate) format: TextureFormat,
}

pub(crate) struct DynamicTextureAtlasBuilder {
    pub atlas_allocator: AtlasAllocator,
    pub padding: i32,
    pub(crate) temp_image_data: TempImageData,
    pub texture_atlas: TextureAtlas,
}

impl DynamicTextureAtlasBuilder {
    pub fn new(size: Vec2, padding: i32, temp_image_data: TempImageData) -> Self {
        let texture_atlas = TextureAtlas::new_empty(size);
        Self {
            atlas_allocator: AtlasAllocator::new(to_size2(size)),
            padding,
            temp_image_data,
            texture_atlas,
        }
    }

    pub(crate) fn add_texture(
        &mut self,
        texture_data_to_add: &TempImageData,
    ) -> Option<(usize, TempImageData)> {
        let allocation = self.atlas_allocator.allocate(size2(
            texture_data_to_add.size.x + self.padding,
            texture_data_to_add.size.y + self.padding,
        ));

        if let Some(allocation) = allocation {
            self.place_texture(allocation, texture_data_to_add);
            let mut rect: Rect = to_rect(allocation.rectangle);
            rect.max -= self.padding as f32;
            Some((
                self.texture_atlas.add_texture(rect),
                self.temp_image_data.clone(),
            ))
        } else {
            None
        }
    }

    fn place_texture(&mut self, allocation: Allocation, texture_data_to_add: &TempImageData) {
        let mut rect = allocation.rectangle;
        rect.max.x -= self.padding;
        rect.max.y -= self.padding;
        let atlas_width = self.temp_image_data.size.x as usize;
        let rect_width = rect.width() as usize;
        let format_size: usize = self.temp_image_data.format.describe().block_size.into();

        for (texture_y, bound_y) in (rect.min.y..rect.max.y).map(|i| i as usize).enumerate() {
            let begin = (bound_y * atlas_width + rect.min.x as usize) * format_size;
            let end = begin + rect_width * format_size;
            let texture_begin = texture_y * rect_width * format_size;
            let texture_end = texture_begin + rect_width * format_size;
            self.temp_image_data.data[begin..end]
                .copy_from_slice(&texture_data_to_add.data[texture_begin..texture_end]);
        }
    }
}

fn to_size2(vec2: Vec2) -> guillotiere::Size {
    guillotiere::Size::new(vec2.x as i32, vec2.y as i32)
}

fn to_rect(rectangle: guillotiere::Rectangle) -> Rect {
    Rect {
        min: IVec2::new(rectangle.min.x, rectangle.min.y).as_vec2(),
        max: IVec2::new(rectangle.max.x, rectangle.max.y).as_vec2(),
    }
}
