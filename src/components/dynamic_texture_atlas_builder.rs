use super::rect::Rect;
use crate::areana::ArenaId;
use guillotiere::{size2, Allocation, AtlasAllocator};

pub struct DynamicTextureAtlasBuilder {
    pub atlas_allocator: AtlasAllocator,
    pub padding: i32,
}

impl DynamicTextureAtlasBuilder {
    pub fn new(size: Vec2, padding: i32) -> Self {
        Self {
            atlas_allocator: AtlasAllocator::new(to_size2(size)),
            padding,
        }
    }

    pub fn add_texture(
        &mut self,
        texture_atlas: &mut TextureAtlas,
        textures: &mut Assets<Image>,
        image: &Image,
    ) -> Option<usize> {
        let allocation = self.atlas_allocator.allocate(size2(
            image.texture_descriptor.size.width as i32 + self.padding,
            image.texture_descriptor.size.height as i32 + self.padding,
        ));

        if let Some(allocation) = allocation {
            let atlas_texture = textures.get_mut(&texture_atlas.texture_id).unwrap();
            self.place_texture(atlas_texture, allocation, image);
            let mut rect: Rect = to_rect(allocation.rectangle);
            rect.max -= self.padding as f32;
            Some(texture_atlas.add_texture(rect))
        } else {
            None
        }
    }

    fn place_texture(&mut self, atlas_image: &mut Image, allocation: Allocation, image: &Image) {
        let mut rect = allocation.rectangle;
        rect.max.x -= self.padding;
        rect.max.y -= self.padding;
        let atlas_width = atlas_image.texture_descriptor.size.width as usize;
        let rect_width = rect.width() as usize;
        let format_size: usize = atlas_image
            .texture_descriptor
            .format
            .describe()
            .block_size
            .into();

        for (texture_y, bound_y) in (rect.min.y..rect.max.y).map(|i| i as usize).enumerate() {
            let begin = (bound_y * atlas_width + rect.min.x as usize) * format_size;
            let end = begin + rect_width * format_size;
            let texture_begin = texture_y * rect_width * format_size;
            let texture_end = texture_begin + rect_width * format_size;
            atlas_image.data[begin..end].copy_from_slice(&image.data[texture_begin..texture_end]);
        }
        atlas_image.dirty = true;
    }
}
