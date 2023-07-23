use asefile::AsepriteFile;
use glam::{Vec2, Vec3, Vec4};
use tobj::LoadOptions;

use crate::arena::ArenaId;
use crate::components::color::Color;
use crate::material::Material;
use crate::mesh::{AttributeValue, Mesh, MeshAttribute, MeshBuilder, Vertex};
use crate::model::Model;
use crate::renderer::{
    fonts::Font,
    texture::{Image, Texture},
};
use image::EncodableLayout;
use std::io::Cursor;
use std::{
    env,
    fs::File,
    future::Future,
    io::Read,
    path::{Path, PathBuf},
    pin::Pin,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Frame {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrameData {
    pub filename: String,
    rotated: bool,
    pub duration: f32,
    pub frame: Frame,
    #[serde(rename(deserialize = "sourceSize"))]
    pub source_size: Size,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Size {
    pub w: i32,
    pub h: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    size: Size,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AsepriteJsonOutput {
    pub frames: Vec<FrameData>,
    pub meta: Meta,
}

/// An owned and dynamically typed Future used when you can't statically type your result or need to add some indirection.
#[cfg(not(target_arch = "wasm32"))]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[cfg(target_arch = "wasm32")]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[cfg(feature = "hot-reloading")]
use crate::file_system_watcher::{AssetType, FilesystemWatcher};

use crate::{audio::AudioSource, Engine};

#[derive(Default)]
pub struct AssetPipeline {
    // pub(crate) textures: Arena<Texture>,
    #[cfg(feature = "hot-reloading")]
    watcher: FilesystemWatcher,
}

impl Engine {
    pub fn load_texture_bytes(&mut self, bytes: &[u8], extension: &str) -> ArenaId<Texture> {
        let image = self
            .asset_pipeline
            .load_texture_from_bytes(bytes, extension)
            .unwrap();

        self.renderer.add_texture(image)
    }

    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<u8>, String> {
        self.asset_pipeline.load_path(path.as_ref())
    }

    pub fn load_aseprite_data_files<P: AsRef<Path>>(
        &mut self,
        json_path: P,
        image_path: P,
    ) -> (Vec<FrameData>, ArenaId<Texture>) {
        let json = self.asset_pipeline.load_path(json_path.as_ref()).unwrap();

        let json_slice = json.as_slice();

        let aseprite_output: Result<AsepriteJsonOutput, serde_json::Error> =
            serde_json::from_slice(json_slice);
        let aseprite_json = aseprite_output.unwrap();

        debug_assert_eq!(
            aseprite_json.meta.size.h, aseprite_json.frames[0].source_size.h,
            "We assume that it will be flat when doing the texture atlas"
        );

        let texture = self.load_texture(image_path);

        (aseprite_json.frames, texture)
    }

    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> ArenaId<Texture> {
        match self.asset_pipeline.load_texture(&path) {
            Ok(image) => {
                let id = self.renderer.add_texture(image);

                #[cfg(feature = "hot-reloading")]
                self.asset_pipeline
                    .watch_file(&path, id.into(), AssetType::Texture);

                id
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    pub fn reload_texture(&mut self, absoulte_file: PathBuf, handle: ArenaId<Texture>) {
        let image = self.asset_pipeline.load_texture(&absoulte_file).unwrap();
        self.renderer.replace_image(handle, image);
    }

    #[allow(clippy::collapsible_match)]
    pub fn watch_change(&mut self) {
        #[cfg(feature = "hot-reloading")]
        {
            if let Ok(recv_event) = self.asset_pipeline.watcher.receiver.try_recv() {
                if let Ok(event) = recv_event {
                    // self.reload_texture(event.paths.get(0).unwrap().get, )
                    let pathbuf = event.paths.get(0).unwrap();

                    match self.asset_pipeline.watcher.asset_map.get(pathbuf) {
                        Some((id, asset_type)) => match asset_type {
                            AssetType::Texture => {
                                let id = *id;
                                let id = id.into();
                                self.reload_texture(pathbuf.to_owned(), id)
                            }
                            _ => {}
                        },
                        None => {
                            println!("Path doesn't match, no reload: {:?}", pathbuf)
                        }
                    }
                }
            }
        }
    }

    pub fn load_audio<P: AsRef<Path>>(&mut self, path: P) -> ArenaId<AudioSource> {
        let byte = self.asset_pipeline.load_path(path.as_ref()).unwrap();

        self.audio.add(byte)
    }

    pub fn load_font_as_default<P: AsRef<Path>>(&mut self, path: P) -> ArenaId<Font> {
        let bytes = self.asset_pipeline.load_path(path.as_ref()).unwrap();
        self.load_font_bytes_as_default(&bytes)
    }

    pub fn load_font_bytes_as_default(&mut self, bytes: &[u8]) -> ArenaId<Font> {
        self.renderer.add_font_as_default(bytes).unwrap()
    }
    pub fn load_font_bytes(&mut self, bytes: &[u8]) -> ArenaId<Font> {
        self.renderer.add_font(bytes).unwrap()
    }

    pub fn load_font<P: AsRef<Path>>(&mut self, path: P) -> ArenaId<Font> {
        let bytes = self.asset_pipeline.load_path(path.as_ref()).unwrap();

        self.load_font_bytes(&bytes)
    }

    pub fn load_obj<P: AsRef<Path>>(&mut self, path: P) -> Model {
        let parent_path = &path.as_ref().parent().unwrap();
        let obj_bytes = self
            .asset_pipeline
            .load_path(&path.as_ref())
            .expect("Couldnt load obj");
        // let mut mtl_bytes = self
        //     .load_path(&path.as_ref().join(".mtl"))
        //     .expect("Couldnt load mtl");
        // let file = Cursor::new(obj_bytes);
        // let mtl = tobj::load_mtl_buf(&mut file).unwrap();
        let mut file = Cursor::new(obj_bytes);
        let (models, materials) = tobj::load_obj_buf(
            &mut file,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| {
                let mtl_bytes = self
                    .asset_pipeline
                    .load_path(&parent_path.join(p))
                    .expect("Couldnt load mtl");
                let mut file: Cursor<Vec<u8>> = Cursor::new(mtl_bytes);
                tobj::load_mtl_buf(&mut file)
            },
        )
        .unwrap();

        let mut model = Model::default();

        for mat in materials.unwrap() {
            if let Some(texture_path) = mat.diffuse_texture {
                let texture_handle = self.load_texture(&parent_path.join(texture_path));
                model.texture = texture_handle;
            }
        }

        for m in models {
            for i in 0..m.mesh.positions.len() / 3 {
                model.positions.push(Vec3::new(
                    m.mesh.positions[i * 3],
                    m.mesh.positions[i * 3 + 1],
                    m.mesh.positions[i * 3 + 2],
                ));

                if !m.mesh.vertex_color.is_empty() {
                    model.colors.push(Vec4::new(
                        m.mesh.vertex_color[i * 3],
                        m.mesh.vertex_color[i * 3 + 1],
                        m.mesh.vertex_color[i * 3 + 2],
                        m.mesh.vertex_color[i * 3 + 3],
                    ));
                }

                if !m.mesh.texcoords.is_empty() {
                    model.tex_coords.push(Vec2::new(
                        m.mesh.texcoords[i * 2],
                        m.mesh.texcoords[i * 2 + 1],
                    ));
                }

                if !m.mesh.normals.is_empty() {
                    model.normals.push(Vec3::new(
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ));
                }
            }

            model.indices = m.mesh.indices;
        }

        model
    }
}

impl AssetPipeline {
    #[cfg(feature = "hot-reloading")]
    pub fn watch_file<P: AsRef<Path>, T>(
        &mut self,
        path: P,
        id: ArenaId<T>,
        asset_type: AssetType,
    ) {
        let full_path = get_base_path().join(path);

        #[cfg(feature = "hot-reloading")]
        self.watcher.watch_file(&full_path, id.into(), asset_type);
    }
    #[cfg(target_arch = "wasm32")]
    async fn load_path_async(&self, path: &Path) -> Result<Vec<u8>, String> {
        use js_sys::Uint8Array;
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::Response;
        let path = get_base_path().join(path);
        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_str(path.to_str().unwrap()))
            .await
            .unwrap();
        let resp: Response = resp_value.dyn_into().unwrap();
        let data = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
        let bytes = Uint8Array::new(&data).to_vec();
        Ok(bytes)
    }

    #[cfg(target_arch = "wasm32")]
    fn load_path(&self, path: &Path) -> Result<Vec<u8>, String> {
        // panic!("Not implemented, waiting for async traits")
        todo!()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_path(&self, path: &Path) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        let full_path = get_base_path().join(path);
        match File::open(full_path) {
            Ok(mut file) => {
                file.read_to_end(&mut bytes).unwrap();
            }
            Err(e) => {
                return if e.kind() == std::io::ErrorKind::NotFound {
                    Err("Error not found".to_string())
                } else {
                    Err(e.to_string())
                }
            }
        }
        Ok(bytes)
    }

    pub fn load_texture_from_bytes(
        &mut self,
        file_bytes: &[u8],
        extension: &str,
    ) -> Result<Image, String> {
        let image = match extension {
            "aseprite" => {
                let ase = AsepriteFile::read(file_bytes).unwrap();

                Image {
                    data: ase.frame(0).image().as_bytes().to_vec(),
                    dimensions: (ase.size().0 as u32, ase.size().1 as u32),
                    ..Default::default()
                }
            }
            "jpg" | "jpeg" => {
                match image::load_from_memory_with_format(file_bytes, image::ImageFormat::Jpeg) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        return Ok(Image {
                            data: img.as_bytes().to_vec(),
                            dimensions: (img.width() as _, img.height() as _),
                            ..Default::default()
                        });
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
            _ => match image::load_from_memory_with_format(file_bytes, image::ImageFormat::Png) {
                Ok(img) => {
                    let img = img.to_rgba8();
                    return Ok(Image {
                        data: img.as_bytes().to_vec(),
                        dimensions: (img.width() as _, img.height() as _),
                        ..Default::default()
                    });
                }
                Err(e) => return Err(e.to_string()),
            },
        };
        Ok(image)
    }

    pub fn load_texture<P: AsRef<Path>>(&mut self, path: &P) -> Result<Image, String> {
        let extension = path.as_ref().extension().expect("Missing extension");
        let file_bytes = self.load_path(path.as_ref())?;

        self.load_texture_from_bytes(&file_bytes, extension.to_str().unwrap())
    }
}

pub fn get_base_path() -> PathBuf {
    #[cfg(not(target_arch = "wasm32"))]
    let root = if let Ok(env_bevy_asset_root) = env::var("BEVY_ASSET_ROOT") {
        PathBuf::from(env_bevy_asset_root)
    } else if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        env::current_exe()
            .map(|path| {
                path.parent()
                    .map(|exe_parent_path| exe_parent_path.to_owned())
                    .unwrap()
            })
            .unwrap()
    };

    #[cfg(not(target_arch = "wasm32"))]
    return root.join("assets");

    #[cfg(target_arch = "wasm32")]
    PathBuf::from("assets")
}
