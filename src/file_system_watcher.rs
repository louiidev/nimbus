use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver},
};

use notify::{Config, Error, Event, RecommendedWatcher, Watcher};

use render_buddy::arena::{ArenaId, WeakArenaId};

#[derive(Debug)]
pub enum AssetType {
    Texture,
    Audio,
    Scene,
}

// pub struct FileChange {
//     pub path: PathBuf,
//     pub asset_id: ArenaId,
//     pub asset_type: AssetType,
// }

pub struct FilesystemWatcher {
    pub watcher: RecommendedWatcher,
    pub receiver: Receiver<Result<Event, Error>>,
    pub path_map: HashMap<PathBuf, HashSet<PathBuf>>,
    pub asset_map: HashMap<PathBuf, (WeakArenaId, AssetType)>,
}

impl Default for FilesystemWatcher {
    fn default() -> Self {
        let (sender, receiver) = channel();
        let watcher: RecommendedWatcher = RecommendedWatcher::new(
            move |res| {
                sender.send(res).expect("Watch event send failure.");
            },
            Config::default(),
        )
        .expect("Failed to create filesystem watcher.");
        Self {
            watcher,
            receiver,
            path_map: HashMap::default(),
            asset_map: HashMap::default(),
        }
    }
}

impl FilesystemWatcher {
    pub(crate) fn watch_file<P: AsRef<Path>>(
        &mut self,
        path: &P,
        id: WeakArenaId,
        asset_type: AssetType,
    ) {
        self.asset_map
            .insert(path.as_ref().to_path_buf(), (id, asset_type));
        let _result = self
            .watcher
            .watch(path.as_ref(), notify::RecursiveMode::NonRecursive);
    }
}
