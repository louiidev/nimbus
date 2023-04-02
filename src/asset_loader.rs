use asefile::AsepriteFile;
use image::EncodableLayout;

use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{
    arena::ArenaId,
    file_system_watcher::{AssetType, FilesystemWatcher},
    internal_image::InternalImage,
    Engine,
};

#[derive(Default)]
pub struct AssetPipeline {
    // pub(crate) textures: Arena<Texture>,
    #[cfg(feature = "hot-reloading")]
    watcher: FilesystemWatcher,
}

impl Engine {
    pub fn load_texture<P: AsRef<Path>>(&mut self, file: P) -> ArenaId {
        let actual_path = get_base_path().join(&file);
        let image = self.asset_pipeline.load_texture(&actual_path);
        let id = self.renderer.as_mut().unwrap().load_texture(image);

        self.asset_pipeline
            .watch_file(&actual_path, id, AssetType::Texture);

        id
    }

    pub fn reload_texture(&mut self, absoulte_file: PathBuf, id: ArenaId) {
        let image = self.asset_pipeline.load_texture(&absoulte_file);
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
}

impl AssetPipeline {
    pub fn watch_file<P: AsRef<Path>>(&mut self, file: P, id: ArenaId, asset_type: AssetType) {
        #[cfg(feature = "hot-reloading")]
        self.watcher.watch_file(&file, id, asset_type);
    }

    pub fn load_texture(&mut self, file: &PathBuf) -> InternalImage {
        let extension = file.extension().expect("Missing extension");
        let bytes = match extension.to_str().unwrap() {
            "asesprite" => {
                let ase = AsepriteFile::read_file(&file).unwrap();

                InternalImage {
                    data: ase.frame(0).image().as_bytes().to_vec(),
                    size: ase.size(),
                }
            }
            _ => {
                let image = image::open(&file).unwrap();
                InternalImage {
                    data: image.as_bytes().to_vec(),
                    size: (image.width() as _, image.height() as _),
                }
            }
        };
        bytes
    }
}

pub fn get_base_path() -> PathBuf {
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        env::current_exe()
            .map(|path| {
                path.parent()
                    .map(|exe_parent_path| exe_parent_path.to_owned())
                    .unwrap()
            })
            .unwrap()
    }
}
