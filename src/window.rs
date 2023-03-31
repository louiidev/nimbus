pub struct WindowDescriptor<'a> {
    pub width: f32,
    pub height: f32,
    pub title: &'a str,
    /// ## Platform-specific
    /// - iOS / Android / Web: Unsupported.
    pub resizable: bool,
}

impl<'a> Default for WindowDescriptor<'a> {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 720.0,
            title: "Nimbus Engine",
            resizable: false,
        }
    }
}
