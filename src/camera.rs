use std::{collections::HashMap, ops::Range, sync::Arc};

use glam::{Mat4, UVec2, Vec2, Vec3};
use wgpu::Extent3d;
use winit::window::Window;

use crate::{
    areana::ArenaId,
    components::{
        ray::{Ray, Ray3D},
        transform::Transform,
    },
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CameraBindGroupType {
    Orthographic,
    OrthographicUI,
    Perspective,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub target: RenderTarget,
    pub target_info: RenderTargetInfo,
    pub(crate) bind_groups: HashMap<CameraBindGroupType, Arc<wgpu::BindGroup>>,
    pub orthographic_projection: OrthographicProjection,
    pub transform: Transform,
}

impl Camera {
    pub fn new_with_far(far: f32, physical_size: UVec2, scale: f32) -> Self {
        // TODO: Make this support any projection and not have a weird hack for UI maybe?
        let mut projection = OrthographicProjection {
            far,
            ..Default::default()
        };

        projection.update(physical_size.x as _, physical_size.y as _);

        Camera {
            orthographic_projection: projection,
            transform: Transform::from_xyz(0.0, 0.0, far - 0.1),
            target_info: RenderTargetInfo {
                physical_size,
                scale_factor: scale,
            },
            ..Default::default()
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target_info: RenderTargetInfo::default(),
            target: Default::default(),
            bind_groups: HashMap::default(),
            orthographic_projection: OrthographicProjection::default(),
            transform: Transform::from_xyz(0.0, 0.0, 1000. - 0.1),
        }
    }
}

// pub fn camera_system(mut camera: Query<(&mut Camera)>) {
//     let mut camera = camera.get_single_mut().unwrap();

//     for event in window_created_events.iter() {
//         camera
//             .orthographic_projection
//             .update(event.width, event.height);

//         camera.computed.target_info = RenderTargetInfo {
//             physical_size: UVec2 {
//                 x: event.width as _,
//                 y: event.height as _,
//             },
//             scale_factor: event.scale,
//         }
//     }

//     for event in window_resized_events.iter() {
//         camera
//             .orthographic_projection
//             .update(event.width, event.height);
//         camera.computed.target_info = RenderTargetInfo {
//             physical_size: UVec2 {
//                 x: event.width as _,
//                 y: event.height as _,
//             },
//             scale_factor: camera.computed.target_info.scale_factor,
//         }
//     }
// }

/// The "target" that a [`Camera`] will render to. For example, this could be a [`Window`](bevy_window::Window)
/// swapchain or an [`Image`].
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RenderTarget {
    /// Window to which the camera's view is rendered.
    #[default]
    Window,
    /// Image to which the camera's view is rendered.
    Image(ArenaId),
}

impl RenderTarget {
    // pub fn get_render_target_info(
    //     &self,
    //     window: &Window,
    //     images: &Assets<Image>,
    // ) -> RenderTargetInfo {
    //     match self {
    //         RenderTarget::Window => RenderTargetInfo {
    //             physical_size: UVec2::new(window.physical_size.x, window.physical_size.y),
    //             scale_factor: window.scale,
    //         },
    //         RenderTarget::Image(image_handle) => {
    //             let image = images.get(image_handle).expect("Error missing image");
    //             let Extent3d { width, height, .. } = image.texture_descriptor.size;
    //             RenderTargetInfo {
    //                 physical_size: UVec2::new(width, height),
    //                 scale_factor: 1.0,
    //             }
    //         }
    //     }
    // }
}

#[derive(Default, Debug, Clone)]
pub struct RenderTargetInfo {
    /// The physical size of this render target (ignores scale factor).
    pub physical_size: UVec2,
    /// The scale factor of this render target.
    pub scale_factor: f32,
}

#[derive(Default, Debug, Clone)]
pub struct ComputedCameraValues {
    target_info: RenderTargetInfo,
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

#[derive(Debug, Clone)]
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
            self.near,
            self.far,
        )
    }

    fn get_projection_matrix_ui(logical_size: Vec2) -> Mat4 {
        Mat4::orthographic_rh(0.0, logical_size.x, logical_size.y, 0.0, 0.0, 1000.)
    }

    fn update(&mut self, width: f32, height: f32) {
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

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Camera {
    #[inline]
    pub fn projection_matrix(&self) -> Mat4 {
        self.orthographic_projection.get_projection_matrix()
    }

    pub fn projection_matrix_ui(&self, logical_window_size: Vec2) -> Mat4 {
        OrthographicProjection::get_projection_matrix_ui(logical_window_size)
        // self.orthographic_projection.get_projection_matrix()
    }

    /// Converts a physical size in this `Camera` to a logical size.
    #[inline]
    pub fn to_logical(&self, physical_size: UVec2) -> Vec2 {
        physical_size.as_vec2() / self.target_info.scale_factor
    }

    /// The rendered physical bounds (minimum, maximum) of the camera. If the `viewport` field is
    /// set to [`Some`], this will be the rect of that custom viewport. Otherwise it will default to
    /// the full physical rect of the current [`RenderTarget`].
    #[inline]
    pub fn physical_viewport_rect(&self) -> (UVec2, UVec2) {
        let min = UVec2::ZERO;
        let max = min + self.physical_viewport_size();
        (min, max)
    }

    /// The rendered logical bounds (minimum, maximum) of the camera. If the `viewport` field is set
    /// to [`Some`], this will be the rect of that custom viewport. Otherwise it will default to the
    /// full logical rect of the current [`RenderTarget`].
    #[inline]
    pub fn logical_viewport_rect(&self) -> (Vec2, Vec2) {
        let (min, max) = self.physical_viewport_rect();
        (self.to_logical(min), self.to_logical(max))
    }

    /// The logical size of this camera's viewport. If the `viewport` field is set to [`Some`], this
    /// will be the size of that custom viewport. Otherwise it will default to the full logical size
    /// of the current [`RenderTarget`].
    ///  For logic that requires the full logical size of the
    /// [`RenderTarget`], prefer [`Camera::logical_target_size`].
    #[inline]
    pub fn logical_viewport_size(&self) -> Vec2 {
        self.to_logical(self.target_info.physical_size)
    }

    /// The full logical size of this camera's [`RenderTarget`], ignoring custom `viewport` configuration.
    /// Note that if the `viewport` field is [`Some`], this will not represent the size of the rendered area.
    /// For logic that requires the size of the actually rendered area, prefer [`Camera::logical_viewport_size`].
    #[inline]
    pub fn logical_target_size(&self) -> Vec2 {
        self.to_logical(self.target_info.physical_size)
    }

    /// The physical size of this camera's viewport. If the `viewport` field is set to [`Some`], this
    /// will be the size of that custom viewport. Otherwise it will default to the full physical size of
    /// the current [`RenderTarget`].
    /// For logic that requires the full physical size of the [`RenderTarget`], prefer [`Camera::physical_target_size`].
    #[inline]
    pub fn physical_viewport_size(&self) -> UVec2 {
        self.target_info.physical_size
    }

    /// The full physical size of this camera's [`RenderTarget`], ignoring custom `viewport` configuration.
    /// Note that if the `viewport` field is [`Some`], this will not represent the size of the rendered area.
    /// For logic that requires the size of the actually rendered area, prefer [`Camera::physical_viewport_size`].
    #[inline]
    pub fn physical_target_size(&self) -> UVec2 {
        self.target_info.physical_size
    }

    /// Returns a ray originating from the camera, that passes through everything beyond `viewport_position`.
    ///
    /// The resulting ray starts on the near plane of the camera.
    ///
    /// If the camera's projection is orthographic the direction of the ray is always equal to `camera_transform.forward()`.
    ///
    /// To get the world space coordinates with Normalized Device Coordinates, you should use
    /// [`ndc_to_world`](Self::ndc_to_world).
    pub fn viewport_to_world(
        &self,
        camera_transform: &Transform,
        viewport_position: Vec2,
    ) -> Option<Ray3D> {
        let target_size = self.logical_viewport_size();
        let ndc = viewport_position * 2. / target_size - Vec2::ONE;
        let projection = self.projection_matrix();

        let ndc_to_world = camera_transform.compute_matrix() * projection.inverse();
        let world_near_plane = ndc_to_world.project_point3(ndc.extend(1.));
        // Using EPSILON because an ndc with Z = 0 returns NaNs.
        let world_far_plane = ndc_to_world.project_point3(ndc.extend(f32::EPSILON));
        (!world_near_plane.is_nan() && !world_far_plane.is_nan()).then_some(Ray3D {
            origin: world_near_plane,
            direction: (world_far_plane - world_near_plane).normalize(),
        })
    }
}
