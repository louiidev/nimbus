use glam::{IVec2, Vec2};

use crate::{internal_image::Image, rect::Rect, resources::utils::Assets};

#[derive(Clone)]
pub struct TextureAtlas {
    texture_id: uuid::Uuid,
    /// The specific areas of the atlas where each texture can be found
    pub textures: Vec<Rect>,
    pub size: Vec2,
}

impl TextureAtlas {
    pub fn new_empty(texture_id: uuid::Uuid, size: Vec2) -> Self {
        Self {
            texture_id,
            size,
            textures: Vec::new(),
        }
    }

    pub fn from_texture_atlas(
        texture_id: uuid::Uuid,
        tile_size: Vec2,
        columns: usize,
        rows: usize,
        padding: Option<Vec2>,
        offset: Option<Vec2>,
    ) -> Self {
        let padding = padding.unwrap_or_default();
        let offset = offset.unwrap_or_default();
        let mut current_padding = Vec2::ZERO;

        let mut textures = Vec::new();

        for y in 0..rows {
            if y > 0 {
                current_padding.y = padding.y;
            }
            for x in 0..columns {
                if x > 0 {
                    current_padding.x = padding.x;
                }

                let cell = Vec2::new(x as f32, y as f32);
                let rect_min = (tile_size + current_padding) * cell + offset;

                let sprite = Rect {
                    min: rect_min,
                    max: rect_min + tile_size,
                };

                textures.push(sprite);
            }
        }

        let grid_size = Vec2::new(columns as f32, rows as f32);

        TextureAtlas {
            texture_id,
            textures,
            size: ((tile_size + current_padding) * grid_size) - current_padding,
        }
    }

    fn add_texture(&mut self, rect: Rect) -> usize {
        self.textures.push(rect);
        self.textures.len() - 1
    }
}

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

    pub fn finish() {}

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

fn to_rect(rectangle: guillotiere::Rectangle) -> Rect {
    Rect {
        min: IVec2::new(rectangle.min.x, rectangle.min.y).as_vec2(),
        max: IVec2::new(rectangle.max.x, rectangle.max.y).as_vec2(),
    }
}

fn to_size2(vec2: Vec2) -> guillotiere::Size {
    guillotiere::Size::new(vec2.x as i32, vec2.y as i32)
}
