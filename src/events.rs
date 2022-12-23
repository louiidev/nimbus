use std::marker::PhantomData;

use bevy_ecs::system::Resource;
use winit::event::VirtualKeyCode;

pub trait Event: Send + Sync + 'static {}
impl<T> Event for T where T: Send + Sync + 'static {}

/// A window event that is sent whenever a window's logical size has changed.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowResized {
    /// The new logical width of the window.
    pub width: f32,
    /// The new logical height of the window.
    pub height: f32,
}

pub struct WindowCreated {
    /// The new logical width of the window.
    pub width: f32,
    /// The new logical height of the window.
    pub height: f32,
}

pub struct KeyboardInput {
    pub state: winit::event::ElementState,
    pub key_code: VirtualKeyCode,
}

#[derive(Debug, Resource)]
pub struct Events<E: Event> {
    /// Holds the oldest still active events.
    /// Note that a.start_event_count + a.len() should always === events_b.start_event_count.
    events_a: EventSequence<E>,
    /// Holds the newer events.
    events_b: EventSequence<E>,
    event_count: usize,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EventId<E: Event> {
    pub id: usize,
    _marker: PhantomData<E>,
}

#[derive(Debug)]
struct EventInstance<E: Event> {
    pub event_id: EventId<E>,
    pub event: E,
}

#[derive(Debug)]
struct EventSequence<E: Event> {
    events: Vec<EventInstance<E>>,
    start_event_count: usize,
}
