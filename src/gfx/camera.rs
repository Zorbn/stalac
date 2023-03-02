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
    projection: Box<dyn CameraProjection>,
    uniform: CameraUniform,
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(device: &wgpu::Device, projection: Box<dyn CameraProjection>) -> Self {
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(projection.as_ref());

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
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.projection.resize(width, height);
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.uniform.update_view_proj(self.projection.as_ref());
        queue.write_buffer(&self.buffer, 0, to_bytes(&[self.uniform]));
    }

    pub fn get_direction_vec(direction: f32) -> cgmath::Vector3<f32> {
        cgmath::vec3(
            direction.to_radians().sin(),
            0.0,
            direction.to_radians().cos(),
        )
    }

    pub fn rotate(&mut self, look_x: f32, look_y: f32) {
        self.projection.rotate(look_x, look_y);
    }

    pub fn position(&self) -> cgmath::Vector3<f32> {
        self.projection.position()
    }

    pub fn teleport(&mut self, position: cgmath::Vector3<f32>) {
        self.projection.teleport(position);
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub trait CameraProjection {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32>;
    fn resize(&mut self, width: u32, height: u32);
    fn rotate(&mut self, look_x: f32, look_y: f32);
    fn position(&self) -> cgmath::Vector3<f32>;
    fn teleport(&mut self, position: cgmath::Vector3<f32>);
}

fn get_aspect(width: u32, height: u32) -> f32 {
    width as f32 / height as f32
}

pub fn get_look_direction(look_x: f32, look_y: f32) -> cgmath::Vector3<f32> {
    let y_rot = cgmath::Matrix3::from_angle_y(cgmath::Deg(look_y));
    let x_rot = cgmath::Matrix3::from_angle_x(cgmath::Deg(look_x));
    let rot = y_rot * x_rot;

    rot * cgmath::Vector3::unit_z()
}

pub struct CameraPerspectiveProjection {
    eye: cgmath::Vector3<f32>,
    look: cgmath::Vector3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
    look_x: f32,
    look_y: f32,
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
            aspect: get_aspect(width, height),
            fov_y,
            z_near,
            z_far,
            look_x: 0.0,
            look_y: 0.0,
        }
    }
}

impl CameraProjection for CameraPerspectiveProjection {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let eye_point = cgmath::point3(self.eye.x, self.eye.y, self.eye.z);
        let target_point = cgmath::point3(
            self.eye.x + self.look.x,
            self.eye.y + self.look.y,
            self.eye.z + self.look.z,
        );
        let view = cgmath::Matrix4::look_at_rh(eye_point, target_point, self.up);
        let proj = cgmath::perspective(
            cgmath::Deg(self.fov_y),
            self.aspect,
            self.z_near,
            self.z_far,
        );

        proj * view
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.aspect = get_aspect(width, height);
    }

    fn rotate(&mut self, look_x: f32, look_y: f32) {
        self.look_x = look_x;
        self.look_y = look_y;
        self.look = get_look_direction(self.look_x, self.look_y);
    }

    fn position(&self) -> cgmath::Vector3<f32> {
        self.eye
    }

    fn teleport(&mut self, position: cgmath::Vector3<f32>) {
        self.eye = position;
    }
}

pub struct CameraOrthographicProjection {
    z_near: f32,
    z_far: f32,
    width: u32,
    height: u32,
    scale: f32,
}

impl CameraOrthographicProjection {
    pub fn new(z_near: f32, z_far: f32, width: u32, height: u32, scale: f32) -> Self {
        Self {
            z_near,
            z_far,
            width,
            height,
            scale,
        }
    }
}

impl CameraProjection for CameraOrthographicProjection {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let eye_point = cgmath::point3(0.0, 0.0, 1.0);
        let target_point = cgmath::point3(0.0, 0.0, 0.0);
        let view = cgmath::Matrix4::look_at_rh(eye_point, target_point, cgmath::Vector3::unit_y());

        cgmath::ortho(
            0.0,
            self.width as f32 / self.scale,
            0.0,
            self.height as f32 / self.scale,
            self.z_near,
            self.z_far,
        ) * view
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn rotate(&mut self, _look_x: f32, _look_y: f32) {}

    fn position(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::zero()
    }

    fn teleport(&mut self, _position: cgmath::Vector3<f32>) {}
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

    pub fn update_view_proj(&mut self, projection: &dyn CameraProjection) {
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * projection.build_view_projection_matrix()).into();
    }
}
