use std::f32::consts::FRAC_PI_2;

#[cfg(feature = "egui")]
use egui_inspect::EguiInspect;
use glam::{Mat4, Quat, Vec2, Vec3};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Device};

use crate::{components::ray::Raycast3D, Engine, Rect, Transform};

#[derive(Default)]
pub struct Camera {
    pub projection: Projection,
    pub transform: Transform,
}

impl EguiInspect for Camera {
    fn inspect(&self, _label: &str, _ui: &mut egui::Ui) {}

    fn inspect_mut(&mut self, _label: &str, ui: &mut egui::Ui) {
        egui::Grid::new("my_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.heading("Camera");
                ui.end_row();
                ui.label("Position");
                ui.end_row();
                ui.label("X:");
                ui.add(egui::DragValue::new(&mut self.transform.position.x).speed(1.));
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut self.transform.position.y).speed(1.));
                ui.label("Z:");
                ui.add(egui::DragValue::new(&mut self.transform.position.z).speed(1.));
                ui.end_row();
                ui.label("Scale:");
                ui.add(egui::DragValue::new(&mut self.projection.scale).speed(1.));
                ui.end_row();
            });
    }
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Camera {
    /// Sets the projection
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
    }

    // // Sets the projection to perspective
    // pub fn set_projection_perspective(&mut self, vfov: f32, near: f32, aspect_ratio: f32) {
    //     self.set_projection(Projection::Perspective {
    //         vfov,
    //         near,
    //         aspect_ratio,
    //     });
    // }

    // Sets the projection to orthographic
    pub fn set_orthographic_perspective(&mut self) {
        self.set_projection(Projection::default());
    }
    // // Sets the projection to custom Matrix
    // pub fn set_custom_projection(&mut self, projection: Mat4) {
    //     self.set_projection(Projection::Custom(projection));
    // }

    pub fn resize(&mut self, width: f32, height: f32) {
        let Projection {
            near,
            far,
            viewport_origin,
            scaling_mode,
            scale,
            area,
        } = &mut self.projection;

        let (projection_width, projection_height) = match scaling_mode {
            ScalingMode::WindowSize(pixel_scale) => (width / *pixel_scale, height / *pixel_scale),
            ScalingMode::AutoMin {
                min_width,
                min_height,
            } => {
                let min_width = *min_width;
                let min_height = *min_height;

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
                let max_width = *max_width;
                let max_height = *max_height;
                // Compare Pixels of current width and maximal height and Pixels of maximal width with current height.
                // Then use smaller (max_height when true) as what it refers to (height when true) and calculate rest so it can't get over maximum.
                if width * max_height < max_width * height {
                    (width * max_height / height, max_height)
                } else {
                    (max_width, height * max_width / width)
                }
            }
            ScalingMode::FixedVertical(viewport_height) => {
                (width * *viewport_height / height, *viewport_height)
            }
            ScalingMode::FixedHorizontal(viewport_width) => {
                (*viewport_width, height * *viewport_width / width)
            }
            ScalingMode::Fixed { width, height } => (*width, *height),
        };

        let origin_x = projection_width * viewport_origin.x;
        let origin_y = projection_height * viewport_origin.y;

        *area = Rect::new(
            *scale * -origin_x,
            *scale * -origin_y,
            *scale * (projection_width - origin_x),
            *scale * (projection_height - origin_y),
        );
    }
}

impl Camera {
    pub fn calc_view_matrix(&self) -> Mat4 {
        self.compute_projection_matrix() * self.transform.compute_matrix().inverse()
    }

    pub(crate) fn create_bind_group(
        &self,
        device: &Device,
        viewport_size: (u32, u32),
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        let view_matrix = self.calc_view_matrix();

        let camera_uniform = CameraUniform {
            view_proj: view_matrix.to_cols_array_2d(),
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&[camera_uniform]),
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Camera bind group"),
            layout: &bind_group_layout,
        })
    }

    pub(crate) fn compute_projection_matrix(&self) -> Mat4 {
        let Projection {
            near, far, area, ..
        } = &self.projection;

        Mat4::orthographic_rh(
            area.min.x, area.max.x, area.min.y, area.max.y,
            // NOTE: near and far are swapped to invert the depth range from [0,1] to [1,0]
            // This is for interoperability with pipelines using infinite reverse perspective projections.
            *near, *far,
        )
    }

    pub fn viewport_to_world_position(
        &self,
        viewport_position: Vec2,
        viewport_size: (u32, u32),
    ) -> Option<Raycast3D> {
        let target_size = Vec2::new(viewport_size.0 as f32, viewport_size.1 as f32);
        let ndc = viewport_position * 2. / target_size - Vec2::ONE;

        let ndc_to_world =
            self.transform.compute_matrix() * self.compute_projection_matrix().inverse();

        let world_near_plane = ndc_to_world.project_point3(ndc.extend(1.));
        // Using EPSILON because an ndc with Z = 0 returns NaNs.
        let world_far_plane = ndc_to_world.project_point3(ndc.extend(f32::EPSILON));
        (!world_near_plane.is_nan() && !world_far_plane.is_nan()).then_some(Raycast3D {
            origin: world_far_plane,
            direction: (world_far_plane - world_near_plane).normalize(),
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CameraOrigin {
    #[default]
    Center,
    TopLeft,
}

#[derive(Debug, Clone)]
pub enum ScalingMode {
    /// Manually specify the projection's size, ignoring window resizing. The image will stretch.
    /// Arguments are in world units.
    Fixed { width: f32, height: f32 },
    /// Match the viewport size.
    /// The argument is the number of pixels that equals one world unit.
    WindowSize(f32),
    /// Keeping the aspect ratio while the axes can't be smaller than given minimum.
    /// Arguments are in world units.
    AutoMin { min_width: f32, min_height: f32 },
    /// Keeping the aspect ratio while the axes can't be bigger than given maximum.
    /// Arguments are in world units.
    AutoMax { max_width: f32, max_height: f32 },
    /// Keep the projection's height constant; width will be adjusted to match aspect ratio.
    /// The argument is the desired height of the projection in world units.
    FixedVertical(f32),
    /// Keep the projection's width constant; height will be adjusted to match aspect ratio.
    /// The argument is the desired width of the projection in world units.
    FixedHorizontal(f32),
}

#[derive(Debug, Clone)]
pub struct Projection {
    near: f32,
    far: f32,
    viewport_origin: Vec2,
    scaling_mode: ScalingMode,
    pub scale: f32,
    area: Rect,
}

impl Default for Projection {
    fn default() -> Self {
        Projection {
            scale: 5.0,
            near: -100.0,
            far: 1000.0,
            viewport_origin: Vec2::new(0.5, 0.5),
            scaling_mode: ScalingMode::FixedVertical(2.0),
            area: Rect::new(-1.0, -1.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }
}

impl Engine {}
