#[cfg(feature = "egui")]
use egui_inspect::EguiInspect;
use glam::{Mat4, Quat, Vec2, Vec3};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Device};

pub const DEFAULT_ORTHO_CAMERA_DEPTH: f32 = 1000.0;

pub struct Camera {
    pub projection: Projection,
    pub position: Vec3,
    pub rotation: Quat,
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
                ui.label("X Axis:");
                ui.add(egui::DragValue::new(&mut self.position.x).speed(1.));
                ui.end_row();
                ui.label("Y Axis:");
                ui.add(egui::DragValue::new(&mut self.position.y).speed(1.));
                ui.end_row();
                ui.label("Z Axis:");
                ui.add(egui::DragValue::new(&mut self.position.z).speed(1.));
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
    /// Overrides the default camera values
    pub fn set_data(&mut self, camera: Camera) {
        self.projection = camera.projection;
        self.position = camera.position;
        self.rotation = camera.rotation;
    }
    /// Sets the camera position
    /// Remember to set the Z value correctly
    /// Otherwise things might not render correctly
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
    /// Sets the camera rotation
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }
    /// Sets the projection
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
    }
    /// Sets the camera origin, only works for Orthographic projection
    pub fn with_origin(mut self, new_origin: CameraOrigin) -> Self {
        if let Projection::Orthographic { ref mut origin, .. } = self.projection {
            *origin = new_origin;
        }

        self
    }
    // Sets the projection to perspective
    pub fn set_projection_perspective(&mut self, vfov: f32, near: f32, aspect_ratio: f32) {
        self.set_projection(Projection::Perspective {
            vfov,
            near,
            aspect_ratio,
        });
    }

    // Sets the projection to orthographic
    pub fn set_orthographic_perspective(
        &mut self,
        origin: CameraOrigin,
        target_resolution: Option<Vec2>,
    ) {
        self.set_projection(Projection::Orthographic {
            origin,
            target_resolution,
        });
    }
    // Sets the projection to custom Matrix
    pub fn set_custom_projection(&mut self, projection: Mat4) {
        self.set_projection(Projection::Custom(projection));
    }

    pub fn get_current_origin(&mut self) -> CameraOrigin {
        if let Projection::Orthographic { origin, .. } = self.projection {
            return origin;
        }

        CameraOrigin::Center
    }
}

impl Camera {
    pub fn orthographic() -> Self {
        Self {
            projection: Projection::Orthographic {
                origin: CameraOrigin::default(),
                target_resolution: None,
            },
            position: Vec3::new(0., 0., DEFAULT_ORTHO_CAMERA_DEPTH - 0.1),
            rotation: Quat::IDENTITY,
        }
    }

    pub fn perspective(fov_radians: f32, near: f32, aspect_ratio: f32) -> Self {
        Self {
            projection: Projection::Perspective {
                vfov: fov_radians,
                near,
                aspect_ratio,
            },
            position: Vec3::new(0., 0., 5.),
            rotation: Quat::IDENTITY,
        }
    }

    pub(crate) fn create_bind_group(
        &self,
        device: &Device,
        viewport_size: (u32, u32),
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        let projection = self.compute_projection_matrix(viewport_size);

        let additive = if let Projection::Orthographic { origin, .. } = self.projection {
            if let CameraOrigin::TopLeft = origin {
                Vec3::new(
                    viewport_size.0 as f32 / 2.,
                    -(viewport_size.1 as f32) / 2.,
                    0.,
                )
            } else {
                Vec3::ZERO
            }
        } else {
            Vec3::ZERO
        };

        let view = Mat4::from_scale_rotation_translation(
            Vec3::splat(1.),
            self.rotation,
            self.position + additive,
        );
        let inverse_view = view.inverse();
        let view_projection = projection * inverse_view;

        let camera_uniform = CameraUniform {
            view_proj: view_projection.to_cols_array_2d(),
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

    pub(crate) fn compute_projection_matrix(&self, viewport_size: (u32, u32)) -> Mat4 {
        match &self.projection {
            Projection::Orthographic {
                target_resolution, ..
            } => {
                let (width, height) = if let Some(Vec2 {
                    x: target_width,
                    y: target_height,
                }) = target_resolution.to_owned()
                {
                    let (width, height) = viewport_size;
                    let width = width as f32;
                    let height = height as f32;
                    if width * target_height < target_width * height {
                        (width * target_height / height, target_height)
                    } else {
                        (target_width, height * target_width / width)
                    }
                } else {
                    (viewport_size.0 as f32, viewport_size.1 as f32)
                };

                let near = DEFAULT_ORTHO_CAMERA_DEPTH / 2.0;
                let half_width = width / 2.0;
                let half_height = height / 2.0;

                Mat4::orthographic_rh(
                    -half_width,
                    half_width,
                    -half_height,
                    half_height,
                    -near,
                    DEFAULT_ORTHO_CAMERA_DEPTH,
                )
            }
            Projection::Perspective {
                vfov,
                near,
                aspect_ratio,
            } => Mat4::perspective_infinite_reverse_rh(*vfov, *aspect_ratio, *near),
            Projection::Custom(custom) => *custom,
        }
    }

    pub fn viewport_to_world_position(
        &self,
        viewport_position: Vec2,
        viewport_size: (u32, u32),
    ) -> Option<Vec3> {
        let target_size = Vec2::new(viewport_size.0 as f32, viewport_size.1 as f32);
        let ndc = viewport_position * 2. / target_size - Vec2::ONE;
        let projection = self.compute_projection_matrix(viewport_size);

        let view =
            Mat4::from_scale_rotation_translation(Vec3::splat(1.), self.rotation, self.position);

        let ndc_to_world = view * projection.inverse();
        let world_near_plane = ndc_to_world.project_point3(ndc.extend(1.));
        // Using EPSILON because an ndc with Z = 0 returns NaNs.
        let world_far_plane = ndc_to_world.project_point3(ndc.extend(f32::EPSILON));
        (!world_near_plane.is_nan() && !world_far_plane.is_nan()).then_some(world_near_plane)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CameraOrigin {
    #[default]
    Center,
    TopLeft,
}

#[derive(Debug, Clone)]
pub enum Projection {
    Orthographic {
        origin: CameraOrigin,
        target_resolution: Option<Vec2>,
    },
    Perspective {
        /// Vertical field of view in degrees.
        vfov: f32,
        /// Near plane distance. All projection uses a infinite far plane.
        near: f32,
        aspect_ratio: f32,
    },
    Custom(Mat4),
}
