use asefile::AsepriteFile;
use image::EncodableLayout;
use std::{
    env,
    fs::File,
    future::Future,
    io::Read,
    path::{Path, PathBuf},
    pin::Pin,
};

/// An owned and dynamically typed Future used when you can't statically type your result or need to add some indirection.
#[cfg(not(target_arch = "wasm32"))]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[cfg(target_arch = "wasm32")]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[cfg(feature = "hot-reloading")]
use crate::file_system_watcher::{AssetType, FilesystemWatcher};

use crate::{arena::ArenaId, internal_image::InternalImage, Engine};

#[derive(Default)]
pub struct AssetPipeline {
    // pub(crate) textures: Arena<Texture>,
    #[cfg(feature = "hot-reloading")]
    watcher: FilesystemWatcher,
}

impl Engine {
    pub fn load_texture_bytes(&mut self, bytes: &[u8], extension: &str) -> ArenaId {
        let image = self
            .asset_pipeline
            .load_texture_from_bytes(bytes, extension)
            .unwrap();

        self.renderer.as_mut().unwrap().load_texture(image)
    }

    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> ArenaId {
        match self.asset_pipeline.load_texture(&path) {
            Ok(image) => {
                let id = self.renderer.as_mut().unwrap().load_texture(image);

                #[cfg(feature = "hot-reloading")]
                self.asset_pipeline
                    .watch_file(&path, id, AssetType::Texture);

                id
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    pub fn reload_texture(&mut self, absoulte_file: PathBuf, id: ArenaId) {
        let image = self.asset_pipeline.load_texture(&absoulte_file).unwrap();
        self.renderer.as_mut().unwrap().replace_texture(id, image);
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
                            AssetType::Texture => self.reload_texture(pathbuf.to_owned(), *id),
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

    pub fn load_audio<P: AsRef<Path>>(&mut self, path: P) -> ArenaId {
        let byte = self.asset_pipeline.load_path(path.as_ref()).unwrap();

        self.audio.add(byte)
    }
}

impl AssetPipeline {
    #[cfg(feature = "hot-reloading")]
    pub fn watch_file<P: AsRef<Path>>(&mut self, path: P, id: ArenaId, asset_type: AssetType) {
        let full_path = get_base_path().join(path);

        #[cfg(feature = "hot-reloading")]
        self.watcher.watch_file(&full_path, id, asset_type);
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
    ) -> Result<InternalImage, String> {
        let bytes = match extension {
            "aseprite" => {
                let ase = AsepriteFile::read(file_bytes).unwrap();

                InternalImage {
                    data: ase.frame(0).image().as_bytes().to_vec(),
                    size: ase.size(),
                }
            }
            _ => match image::load_from_memory_with_format(file_bytes, image::ImageFormat::Png) {
                Ok(img) => {
                    let img = img.to_rgba8();
                    return Ok(InternalImage {
                        data: img.as_bytes().to_vec(),
                        size: (img.width() as _, img.height() as _),
                    });
                }
                Err(e) => return Err(e.to_string()),
            },
        };
        Ok(bytes)
    }

    pub fn load_texture<P: AsRef<Path>>(&mut self, path: &P) -> Result<InternalImage, String> {
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
