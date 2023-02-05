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
            look_y: 180.0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.projection.aspect = CameraPerspectiveProjection::get_aspect(width, height);
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.uniform.update_view_proj(&self.projection);
        queue.write_buffer(&self.buffer, 0, to_bytes(&[self.uniform]));
    }

    pub fn translate(&mut self, dir_x: f32, dir_y: f32, dir_z: f32, speed: f32) {
        let forward = Self::get_direction_vec(self.look_y);
        let right = Self::get_direction_vec(self.look_y + 90.0);
        let dir = dir_z * forward + dir_x * right + dir_y * cgmath::Vector3::unit_y();

        if dir.magnitude() == 0.0 {
            return;
        }

        let velocity = dir.normalize() * speed;
        self.projection.eye += velocity;
        self.projection.target += velocity;
    }

    pub fn get_direction_vec(direction: f32) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            direction.to_radians().sin(),
            0.0,
            direction.to_radians().cos(),
        )
    }

    pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        self.look_x += delta_x;
        self.look_y += delta_y;
        self.look_x = self.look_x.clamp(-89.0, 89.0);

        let y_rot = cgmath::Matrix3::from_angle_y(cgmath::Deg(self.look_y));
        let x_rot = cgmath::Matrix3::from_angle_x(cgmath::Deg(self.look_x));
        let rot = y_rot * x_rot;
        let new_target = self.projection.eye.to_vec() + rot * cgmath::Vector3::unit_z();
        self.projection.target = cgmath::Point3::new(new_target.x, new_target.y, new_target.z);
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub struct CameraPerspectiveProjection {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl CameraPerspectiveProjection {
    pub fn new(
        position: cgmath::Point3<f32>,
        fov_y: f32,
        z_near: f32,
        z_far: f32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            eye: position,
            target: position - cgmath::Vector3::unit_z(),
            up: cgmath::Vector3::unit_y(),
            aspect: Self::get_aspect(width, height),
            fov_y,
            z_near,
            z_far,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
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
