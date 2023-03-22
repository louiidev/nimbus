#[derive(Default)]
pub struct WindowDescriptor<'a> {
    pub width: f32,
    pub height: f32,
    pub title: &'a str,
    /// ## Platform-specific
    /// - iOS / Android / Web: Unsupported.
    pub resizable: bool,
}
