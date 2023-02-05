use cgmath::prelude::*;
use wgpu::util::DeviceExt;

use crate::bytes::to_bytes;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    projection: CameraPerspectiveProjection,
    uniform: CameraUniform,
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    look_x: f32,
    look_y: f32,
}

impl Camera {
    pub fn new(device: &wgpu::Device, projection: CameraPerspectiveProjection) -> Self {
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&projection);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: to_bytes(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            projection,
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
            look_x: 0.0,
            look_y: 0.0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.projection.aspect = CameraPerspectiveProjection::get_aspect(width, height);
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.uniform.update_view_proj(&self.projection);
        queue.write_buffer(&self.buffer, 0, to_bytes(&[self.uniform]));
    }

    pub fn get_direction_vec(direction: f32) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            direction.to_radians().sin(),
            0.0,
            direction.to_radians().cos(),
        )
    }

    pub fn rotate(&mut self, look_x: f32, look_y: f32) {
        self.look_x = look_x;
        self.look_y = look_y;

        let y_rot = cgmath::Matrix3::from_angle_y(cgmath::Deg(self.look_y));
        let x_rot = cgmath::Matrix3::from_angle_x(cgmath::Deg(self.look_x));
        let rot = y_rot * x_rot;
        self.projection.look = rot * cgmath::Vector3::unit_z();
    }

    pub fn position(&self) -> cgmath::Vector3<f32> {
        self.projection.eye
    }

    pub fn teleport(&mut self, position: cgmath::Vector3<f32>) {
        self.projection.eye = position;
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub struct CameraPerspectiveProjection {
    pub eye: cgmath::Vector3<f32>,
    pub look: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl CameraPerspectiveProjection {
    pub fn new(
        position: cgmath::Vector3<f32>,
        fov_y: f32,
        z_near: f32,
        z_far: f32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            eye: position,
            look: cgmath::Vector3::unit_z(),
            up: cgmath::Vector3::unit_y(),
            aspect: Self::get_aspect(width, height),
            fov_y,
            z_near,
            z_far,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let eye_point = cgmath::Point3::new(self.eye.x, self.eye.y, self.eye.z);
        let target_point = cgmath::Point3::new(self.eye.x + self.look.x, self.eye.y + self.look.y, self.eye.z + self.look.z);
        let view = cgmath::Matrix4::look_at_rh(eye_point, target_point, self.up);
        let proj = cgmath::perspective(
            cgmath::Deg(self.fov_y),
            self.aspect,
            self.z_near,
            self.z_far,
        );

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn get_aspect(width: u32, height: u32) -> f32 {
        width as f32 / height as f32
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &CameraPerspectiveProjection) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
