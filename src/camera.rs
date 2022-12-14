use std::{ops::Range, sync::Arc};

use bevy_ecs::{
    bundle,
    prelude::{Bundle, Component, EventReader},
    system::{Query, Res, ResMut, Resource},
};
use glam::{Mat4, UVec2, Vec2};

use crate::{
    events::{WindowCreated, WindowResized},
    transform::{GlobalTransform, Transform},
};

#[derive(Bundle, Debug)]
pub struct CameraBundle {
    pub camera: Camera,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for CameraBundle {
    fn default() -> Self {
        Self::new_with_far(1000.0)
    }
}

impl CameraBundle {
    /// Create an orthographic projection camera with a custom `Z` position.
    ///
    /// The camera is placed at `Z=far-0.1`, looking toward the world origin `(0,0,0)`.
    /// Its orthographic projection extends from `0.0` to `-far` in camera view space,
    /// corresponding to `Z=far-0.1` (closest to camera) to `Z=-0.1` (furthest away from
    /// camera) in world space.
    pub fn new_with_far(far: f32) -> Self {
        // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
        // the camera's translation by far and use a right handed coordinate system

        let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
        Self {
            transform,
            global_transform: Default::default(),
            camera: Camera::new_with_far(far),
        }
    }
}

pub fn camera_system(
    mut window_resized_events: EventReader<WindowResized>,
    mut window_created_events: EventReader<WindowCreated>,
    mut camera: Query<(&mut Camera)>,
) {
    let mut camera = camera.get_single_mut().unwrap();

    for event in window_created_events.iter() {
        camera
            .orthographic_projection
            .update(event.width, event.height)
    }

    for event in window_resized_events.iter() {
        camera
            .orthographic_projection
            .update(event.width, event.height)
    }
}

/// The "target" that a [`Camera`] will render to. For example, this could be a [`Window`](bevy_window::Window)
/// swapchain or an [`Image`].
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RenderTarget {
    /// Window to which the camera's view is rendered.
    #[default]
    Window,
    /// Image to which the camera's view is rendered.
    Image(uuid::Uuid),
}

#[derive(Debug, Clone)]
pub struct Viewport {
    /// The physical position to render this viewport to within the [`RenderTarget`] of this [`Camera`].
    /// (0,0) corresponds to the top-left corner
    pub physical_position: UVec2,
    /// The physical size of the viewport rectangle to render to within the [`RenderTarget`] of this [`Camera`].
    /// The origin of the rectangle is in the top-left corner.
    pub physical_size: UVec2,
    /// The minimum and maximum depth to render (on a scale from 0.0 to 1.0).
    pub depth: Range<f32>,
}

#[derive(Default, Debug, Clone)]
pub struct RenderTargetInfo {
    /// The physical size of this render target (ignores scale factor).
    pub physical_size: UVec2,
    /// The scale factor of this render target.
    pub scale_factor: f64,
}

#[derive(Default, Debug, Clone)]
pub struct ComputedCameraValues {
    projection_matrix: Mat4,
    target_info: Option<RenderTargetInfo>,
    // position and size of the `Viewport`
    old_viewport_size: Option<UVec2>,
}

#[derive(Debug, Clone)]
pub enum ScalingMode {
    /// Manually specify left/right/top/bottom values.
    /// Ignore window resizing; the image will stretch.
    None,
    /// Match the window size. 1 world unit = 1 pixel.
    WindowSize,
    /// Keeping the aspect ratio while the axes can't be smaller than given minimum.
    /// Arguments are in world units.
    AutoMin { min_width: f32, min_height: f32 },
    /// Keeping the aspect ratio while the axes can't be bigger than given maximum.
    /// Arguments are in world units.
    AutoMax { max_width: f32, max_height: f32 },
    /// Keep vertical axis constant; resize horizontal with aspect ratio.
    /// The argument is the desired height of the viewport in world units.
    FixedVertical(f32),
    /// Keep horizontal axis constant; resize vertical with aspect ratio.
    /// The argument is the desired width of the viewport in world units.
    FixedHorizontal(f32),
}

#[derive(Debug, Clone)]
pub enum WindowOrigin {
    Center,
    TopLeft,
}

#[derive(Component, Debug, Clone)]
pub struct OrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
    pub window_origin: WindowOrigin,
    pub scaling_mode: ScalingMode,
    pub scale: f32,
}

impl OrthographicProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left * self.scale,
            self.right * self.scale,
            self.bottom * self.scale,
            self.top * self.scale,
            // NOTE: near and far are swapped to invert the depth range from [0,1] to [1,0]
            // This is for interoperability with pipelines using infinite reverse perspective projections.
            self.far,
            self.near,
        )
    }

    fn update(&mut self, width: f32, height: f32) {
        println!("UPDATES");
        let (viewport_width, viewport_height) = match self.scaling_mode {
            ScalingMode::WindowSize => (width, height),
            ScalingMode::AutoMin {
                min_width,
                min_height,
            } => {
                // Compare Pixels of current width and minimal height and Pixels of minimal width with current height.
                // Then use bigger (min_height when true) as what it refers to (height when true) and calculate rest so it can't get under minimum.
                if width * min_height > min_width * height {
                    (width * min_height / height, min_height)
                } else {
                    (min_width, height * min_width / width)
                }
            }
            ScalingMode::AutoMax {
                max_width,
                max_height,
            } => {
                // Compare Pixels of current width and maximal height and Pixels of maximal width with current height.
                // Then use smaller (max_height when true) as what it refers to (height when true) and calculate rest so it can't get over maximum.
                if width * max_height < max_width * height {
                    (width * max_height / height, max_height)
                } else {
                    (max_width, height * max_width / width)
                }
            }
            ScalingMode::FixedVertical(viewport_height) => {
                (width * viewport_height / height, viewport_height)
            }
            ScalingMode::FixedHorizontal(viewport_width) => {
                (viewport_width, height * viewport_width / width)
            }
            ScalingMode::None => return,
        };

        match self.window_origin {
            WindowOrigin::Center => {
                let half_width = viewport_width / 2.0;
                let half_height = viewport_height / 2.0;
                self.left = -half_width;
                self.bottom = -half_height;
                self.right = half_width;
                self.top = half_height;

                if let ScalingMode::WindowSize = self.scaling_mode {
                    if self.scale == 1.0 {
                        self.left = self.left.floor();
                        self.bottom = self.bottom.floor();
                        self.right = self.right.floor();
                        self.top = self.top.floor();
                    }
                }
            }
            WindowOrigin::TopLeft => {
                self.left = 0.0;
                self.top = 0.0;
                self.right = viewport_width;
                self.bottom = viewport_height;
            }
        }

        dbg!(self);
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        OrthographicProjection {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
            near: 0.0,
            far: 1000.0,
            window_origin: WindowOrigin::Center,
            scaling_mode: ScalingMode::WindowSize,
            scale: 1.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Camera {
    pub viewport: Option<Viewport>,
    pub is_active: bool,
    pub target: RenderTarget,
    pub computed: ComputedCameraValues,
    pub(crate) bind_group: Option<Arc<wgpu::BindGroup>>,
    pub orthographic_projection: OrthographicProjection,
}

impl Camera {
    pub fn new_with_far(far: f32) -> Self {
        Camera {
            orthographic_projection: OrthographicProjection {
                far,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            is_active: true,
            viewport: None,
            computed: Default::default(),
            target: Default::default(),
            bind_group: None,
            orthographic_projection: OrthographicProjection::default(),
        }
    }
}

impl Camera {
    #[inline]
    pub fn projection_matrix(&self) -> Mat4 {
        self.orthographic_projection.get_projection_matrix()
    }

    /// Converts a physical size in this `Camera` to a logical size.
    #[inline]
    pub fn to_logical(&self, physical_size: UVec2) -> Option<Vec2> {
        let scale = self.computed.target_info.as_ref()?.scale_factor;
        Some((physical_size.as_dvec2() / scale).as_vec2())
    }

    /// The rendered physical bounds (minimum, maximum) of the camera. If the `viewport` field is
    /// set to [`Some`], this will be the rect of that custom viewport. Otherwise it will default to
    /// the full physical rect of the current [`RenderTarget`].
    #[inline]
    pub fn physical_viewport_rect(&self) -> (UVec2, UVec2) {
        let min = self
            .viewport
            .as_ref()
            .map(|v| v.physical_position)
            .unwrap_or(UVec2::ZERO);
        let max = min + self.physical_viewport_size().unwrap_or(UVec2::ZERO);
        (min, max)
    }

    /// The rendered logical bounds (minimum, maximum) of the camera. If the `viewport` field is set
    /// to [`Some`], this will be the rect of that custom viewport. Otherwise it will default to the
    /// full logical rect of the current [`RenderTarget`].
    #[inline]
    pub fn logical_viewport_rect(&self) -> Option<(Vec2, Vec2)> {
        let (min, max) = self.physical_viewport_rect();
        Some((self.to_logical(min)?, self.to_logical(max)?))
    }

    /// The physical size of this camera's viewport. If the `viewport` field is set to [`Some`], this
    /// will be the size of that custom viewport. Otherwise it will default to the full physical size of
    /// the current [`RenderTarget`].
    /// For logic that requires the full physical size of the [`RenderTarget`], prefer [`Camera::physical_target_size`].
    #[inline]
    pub fn physical_viewport_size(&self) -> Option<UVec2> {
        self.viewport
            .as_ref()
            .map(|v| v.physical_size)
            .or_else(|| self.physical_target_size())
    }

    /// The full physical size of this camera's [`RenderTarget`], ignoring custom `viewport` configuration.
    /// Note that if the `viewport` field is [`Some`], this will not represent the size of the rendered area.
    /// For logic that requires the size of the actually rendered area, prefer [`Camera::physical_viewport_size`].
    #[inline]
    pub fn physical_target_size(&self) -> Option<UVec2> {
        self.computed.target_info.as_ref().map(|t| t.physical_size)
    }
}
