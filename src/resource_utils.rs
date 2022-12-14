use bevy_ecs::system::Resource;
use std::collections::HashMap;

#[derive(Debug, Default, Resource)]
pub struct Asset<T> {
    pub data: HashMap<uuid::Uuid, T>,
}

impl<T> Asset<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

#[derive(Resource)]
pub struct ResourceVec<T> {
    pub value: Vec<T>,
}

impl<T> ResourceVec<T> {
    pub fn new(value: Vec<T>) -> Self {
        Self { value }
    }
}
