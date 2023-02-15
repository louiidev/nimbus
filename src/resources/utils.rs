use bevy_ecs::system::Resource;
use std::collections::HashMap;

#[derive(Debug, Default, Resource)]
pub struct Assets<T> {
    pub data: HashMap<uuid::Uuid, T>,
}

impl<T> Assets<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get_mut(&mut self, id: &uuid::Uuid) -> Option<&mut T> {
        self.data.get_mut(id)
    }

    pub fn get(&self, id: &uuid::Uuid) -> Option<&T> {
        self.data.get(id)
    }

    pub fn insert(&mut self, id: uuid::Uuid, value: T) -> Option<T> {
        self.data.insert(id, value)
    }

    pub fn add(&mut self, value: T) -> uuid::Uuid {
        let id = uuid::Uuid::new_v4();
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
