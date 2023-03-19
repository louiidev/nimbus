use bevy_ecs::system::Resource;

#[derive(Debug, PartialEq, Eq)]
pub enum EditorMode {
    Game,
    Editor,
}

impl Default for EditorMode {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        return EditorMode::Editor;

        #[cfg(not(debug_assertions))]
        EditorMode::Game
    }
}

#[derive(Default, Debug, Resource)]
pub struct Editor {
    pub mode: EditorMode,
}

impl Editor {
    pub fn toggle(&mut self) {
        self.mode = if self.mode == EditorMode::Game {
            EditorMode::Editor
        } else {
            EditorMode::Game
        };
    }
}

impl Editor {}
