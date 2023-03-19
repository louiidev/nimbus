use bevy_ecs::system::Resource;
use std::collections::HashMap;

#[derive(Debug, Default, Resource)]
pub struct Assets<T> {
    pub data: HashMap<ArenaId, T>,
}

impl<T> Assets<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get_mut(&mut self, id: &ArenaId) -> Option<&mut T> {
        self.data.get_mut(id)
    }

    pub fn get(&self, id: &ArenaId) -> Option<&T> {
        self.data.get(id)
    }

    pub fn insert(&mut self, id: ArenaId, value: T) -> Option<T> {
        self.data.insert(id, value)
    }

    pub fn add(&mut self, value: T) -> ArenaId {
        let id = ArenaId::new_v4();
        self.data.insert(id, value);

        id
    }
}

#[derive(Resource)]
pub struct ResourceVec<T> {
    pub values: Vec<T>,
}

impl<T> ResourceVec<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self { values }
    }
}
