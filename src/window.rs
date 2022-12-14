use std::default::{self};

use glam::Vec2;
use log::warn;
use wgpu::PresentMode;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};

pub fn create_window(window_descriptor: WindowDescriptor) -> (Window, EventLoop<()>) {
    let WindowDescriptor {
        width,
        height,
        position,
        scale_factor_override,
        ..
    } = window_descriptor;

    let logical_size = LogicalSize::new(width, height);

    let mut builder = WindowBuilder::new();

    builder = match window_descriptor.mode {
        WindowMode::BorderlessFullscreen => {
            builder.with_fullscreen(Some(Fullscreen::Borderless(None)))
        }
        WindowMode::Fullscreen => builder.with_fullscreen(None),
        WindowMode::SizedFullscreen => builder.with_fullscreen(None),
        _ => {
            if let Some(sf) = scale_factor_override {
                builder.with_inner_size(logical_size.to_physical::<f64>(sf))
            } else {
                builder.with_inner_size(logical_size)
            }
        }
        .with_resizable(window_descriptor.resizable)
        .with_decorations(window_descriptor.decorations)
        .with_transparent(window_descriptor.transparent)
        .with_always_on_top(window_descriptor.always_on_top),
    };

    let constraints = window_descriptor.resize_constraints.check_constraints();
    let min_inner_size = LogicalSize {
        width: constraints.min_width,
        height: constraints.min_height,
    };
    let max_inner_size = LogicalSize {
        width: constraints.max_width,
        height: constraints.max_height,
    };

    builder = if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
        builder
            .with_min_inner_size(min_inner_size)
            .with_max_inner_size(max_inner_size)
    } else {
        builder.with_min_inner_size(min_inner_size)
    };

    builder = builder.with_title(&window_descriptor.title);

    let event_loop = EventLoop::new();
    let window = builder.build(&event_loop).unwrap();

    (window, event_loop)
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowDescriptor {
    /// The requested logical width of the window's client area.
    ///
    /// May vary from the physical width due to different pixel density on different monitors.
    pub width: f32,
    /// The requested logical height of the window's client area.
    ///
    /// May vary from the physical height due to different pixel density on different monitors.
    pub height: f32,
    /// The position on the screen that the window will be placed at.
    ///
    /// The monitor to place the window on can be selected with the `monitor` field.
    ///
    /// Ignored if `mode` is set to something other than [`WindowMode::Windowed`]
    ///
    /// `WindowPosition::Automatic` will be overridden with `WindowPosition::At(Vec2::ZERO)` if a specific monitor is selected.
    pub position: WindowPosition,
    /// Sets minimum and maximum resize limits.
    pub resize_constraints: WindowResizeConstraints,
    /// Overrides the window's ratio of physical pixels to logical pixels.
    ///
    /// If there are some scaling problems on X11 try to set this option to `Some(1.0)`.
    pub scale_factor_override: Option<f64>,
    /// Sets the title that displays on the window top bar, on the system task bar and other OS specific places.
    ///
    /// ## Platform-specific
    /// - Web: Unsupported.
    pub title: String,
    /// Controls when a frame is presented to the screen.
    #[doc(alias = "vsync")]
    /// The window's [`PresentMode`].
    ///
    /// Used to select whether or not VSync is used
    pub present_mode: PresentMode,
    /// Sets whether the window is resizable.
    ///
    /// ## Platform-specific
    /// - iOS / Android / Web: Unsupported.
    pub resizable: bool,
    /// Sets whether the window should have borders and bars.
    pub decorations: bool,
    /// Sets whether the cursor is visible when the window has focus.
    pub cursor_visible: bool,
    /// Sets whether and how the window grabs the cursor.
    pub cursor_grab_mode: CursorGrabMode,
    /// Sets the [`WindowMode`](crate::WindowMode).
    ///
    /// The monitor to go fullscreen on can be selected with the `monitor` field.
    pub mode: WindowMode,
    /// Sets whether the background of the window should be transparent.
    ///
    /// ## Platform-specific
    /// - iOS / Android / Web: Unsupported.
    /// - macOS: Not working as expected. See [Bevy #6330](https://github.com/bevyengine/bevy/issues/6330).
    /// - Linux (Wayland): Not working as expected. See [Bevy #5779](https://github.com/bevyengine/bevy/issues/5779).
    pub transparent: bool,
    /// The "html canvas" element selector.
    ///
    /// If set, this selector will be used to find a matching html canvas element,
    /// rather than creating a new one.   
    /// Uses the [CSS selector format](https://developer.mozilla.org/en-US/docs/Web/API/Document/querySelector).
    ///
    /// This value has no effect on non-web platforms.
    pub canvas: Option<String>,
    /// Whether or not to fit the canvas element's size to its parent element's size.
    ///
    /// **Warning**: this will not behave as expected for parents that set their size according to the size of their
    /// children. This creates a "feedback loop" that will result in the canvas growing on each resize. When using this
    /// feature, ensure the parent's size is not affected by its children.
    ///
    /// This value has no effect on non-web platforms.
    pub fit_canvas_to_parent: bool,
    /// Sets the window to always be on top of other windows.
    ///
    /// ## Platform-specific
    /// - iOS / Android / Web: Unsupported.
    /// - Linux (Wayland): Unsupported.
    pub always_on_top: bool,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        WindowDescriptor {
            title: "app".to_string(),
            width: 1280.,
            height: 720.,
            position: WindowPosition::Automatic,
            resize_constraints: WindowResizeConstraints::default(),
            scale_factor_override: None,
            present_mode: PresentMode::Fifo,
            resizable: true,
            decorations: true,
            cursor_grab_mode: CursorGrabMode::None,
            cursor_visible: true,
            mode: WindowMode::Windowed,
            transparent: false,
            canvas: None,
            fit_canvas_to_parent: false,
            always_on_top: false,
        }
    }
}

/// The size limits on a window.
///
/// These values are measured in logical pixels, so the user's
/// scale factor does affect the size limits on the window.
/// Please note that if the window is resizable, then when the window is
/// maximized it may have a size outside of these limits. The functionality
/// required to disable maximizing is not yet exposed by winit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowResizeConstraints {
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

impl Default for WindowResizeConstraints {
    fn default() -> Self {
        Self {
            min_width: 180.,
            min_height: 120.,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        }
    }
}

impl WindowResizeConstraints {
    #[must_use]
    pub fn check_constraints(&self) -> Self {
        let WindowResizeConstraints {
            mut min_width,
            mut min_height,
            mut max_width,
            mut max_height,
        } = self;
        min_width = min_width.max(1.);
        min_height = min_height.max(1.);
        if max_width < min_width {
            warn!(
                "The given maximum width {} is smaller than the minimum width {}",
                max_width, min_width
            );
            max_width = min_width;
        }
        if max_height < min_height {
            warn!(
                "The given maximum height {} is smaller than the minimum height {}",
                max_height, min_height
            );
            max_height = min_height;
        }
        WindowResizeConstraints {
            min_width,
            min_height,
            max_width,
            max_height,
        }
    }
}

/// Defines the way a window is displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowMode {
    /// Creates a window that uses the given size.
    #[default]
    Windowed,
    /// Creates a borderless window that uses the full size of the screen.
    BorderlessFullscreen,
    /// Creates a fullscreen window that will render at desktop resolution.
    ///
    /// The app will use the closest supported size from the given size and scale it to fit the screen.
    SizedFullscreen,
    /// Creates a fullscreen window that uses the maximum supported size.
    Fullscreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorGrabMode {
    /// The cursor can freely leave the window.
    #[default]
    None,
    /// The cursor is confined to the window area.
    Confined,
    /// The cursor is locked inside the window area to a certain position.
    Locked,
}

/// Defines where window should be placed at on creation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum WindowPosition {
    /// The position will be set by the window manager.
    #[default]
    Automatic,
    /// Center the window on the monitor.
    ///
    /// The monitor to center the window on can be selected with the `monitor` field in `WindowDescriptor`.
    Centered,
    /// The window's top-left corner will be placed at the specified position in pixels.
    ///
    /// (0,0) represents top-left corner of the monitor.
    ///
    /// The monitor to position the window on can be selected with the `monitor` field in `WindowDescriptor`.
    At(Vec2),
}
