use fontdue::FontResult;

use crate::{
    font::{self, FontData},
    internal_image::Image,
    renderer::{texture::Texture, Renderer},
    resources::utils::Assets,
    App,
};

impl App {
    pub fn load_texture_with_id(&mut self, bytes: &[u8], id: uuid::Uuid) -> uuid::Uuid {
        let renderer = self.world.get_resource::<Renderer>().unwrap();
        let texture = Texture::from_bytes(&renderer.device, &renderer.queue, bytes, "None");

        let mut textures = self.world.get_resource_mut::<Assets<Texture>>().unwrap();

        textures.insert(id, texture);

        id
    }

    pub fn load_texture_with_id_image(&mut self, image: Image, id: uuid::Uuid) -> uuid::Uuid {
        let renderer = self.world.get_resource::<Renderer>().unwrap();
        let texture = Texture::from_image(&renderer.device, &renderer.queue, &image, None);

        let mut textures = self.world.get_resource_mut::<Assets<Texture>>().unwrap();

        textures.insert(id, texture);

        id
    }

    pub fn load_texture(&mut self, bytes: &[u8]) -> uuid::Uuid {
        self.load_texture_with_id(bytes, uuid::Uuid::new_v4())
    }

    pub fn load_font_with_id(
        &mut self,
        font_data: &[u8],
        id: uuid::Uuid,
    ) -> FontResult<uuid::Uuid> {
        let font = font::FontData::try_from_bytes(font_data)?;

        let mut fonts = self.world.get_resource_mut::<Assets<FontData>>().unwrap();
        let _ = fonts.insert(id, font);
        Ok(id)
    }

    pub fn load_font(&mut self, font_data: &[u8]) -> FontResult<uuid::Uuid> {
        self.load_font_with_id(font_data, uuid::Uuid::new_v4())
    }
}
