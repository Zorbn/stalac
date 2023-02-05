use winit::event::VirtualKeyCode;
use cgmath::prelude::*;

use crate::{input::Input, camera::Camera, chunk::Chunk};

const MOUSE_SENSITIVITY: f32 = 0.1;
const GRAVITY: f32 = 30.0;
const JUMP_FORCE: f32 = 9.0;

pub struct Player {
    speed: f32,
    size: cgmath::Vector3<f32>,
    position: cgmath::Vector3<f32>,
    look_x: f32,
    look_y: f32,
    y_velocity: f32,
}

impl Player {
    pub fn new(position: cgmath::Vector3<f32>, size: cgmath::Vector3<f32>, speed: f32) -> Self {
        Self {
            position,
            size,
            speed,
            look_x: 0.0,
            look_y: 0.0,
            y_velocity: 0.0,
        }
    }

    pub fn update(&mut self, input: &mut Input, chunk: &Chunk, delta_time: f32) {
        let mut dir_z = 0.0;
        let mut dir_x = 0.0;

        if input.is_key_held(VirtualKeyCode::W) {
            dir_z += 1.0;
        }

        if input.is_key_held(VirtualKeyCode::S) {
            dir_z -= 1.0;
        }

        if input.is_key_held(VirtualKeyCode::A) {
            dir_x += 1.0;
        }

        if input.is_key_held(VirtualKeyCode::D) {
            dir_x -= 1.0;
        }

        let grounded = chunk.get_block_collision(self.position - cgmath::Vector3::new(0.0, 0.1, 0.0), self.size).is_some();

        // If the player is moving towards the ground while touching it, snap to the floor
        // and prevent y_velocity from building up over time.
        if grounded && self.y_velocity < 0.0 {
            self.y_velocity = 0.0;
            self.position.y = self.position.y.floor() + self.size.y * 0.5;
        }

        self.y_velocity -= GRAVITY * delta_time;

        if grounded && input.is_key_held(VirtualKeyCode::Space) {
            self.y_velocity = JUMP_FORCE;
        }

        let no_clip = input.is_key_held(VirtualKeyCode::V);

        let forward = Camera::get_direction_vec(self.look_y);
        let right = Camera::get_direction_vec(self.look_y + 90.0);
        let mut dir = dir_z * forward + dir_x * right;

        if dir.magnitude() != 0.0 {
            dir = dir.normalize();
        }

        self.step(cgmath::Vector3::new(dir.x, 0.0, 0.0), self.speed * delta_time, chunk, no_clip);
        self.step(cgmath::Vector3::new(0.0, 0.0, dir.z), self.speed * delta_time, chunk, no_clip);

        if !self.step(cgmath::Vector3::unit_y(), self.y_velocity * delta_time, chunk, no_clip) {
            self.y_velocity = 0.0;
        }

        self.rotate(input.mouse_delta_y() * MOUSE_SENSITIVITY, -input.mouse_delta_x() * MOUSE_SENSITIVITY);
    }

    fn step(&mut self, dir: cgmath::Vector3<f32>, speed: f32, chunk: &Chunk, no_clip: bool) -> bool {
        let old_position = self.position;

        let velocity = dir * speed;
        self.position += velocity;

        if !no_clip && chunk.get_block_collision(self.position, self.size).is_some() {
            self.position = old_position;
            return false;
        }

        true
    }

    fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        self.look_x += delta_x;
        self.look_y += delta_y;
        self.look_x = self.look_x.clamp(-89.0, 89.0);
    }

    pub fn teleport(&mut self, position: cgmath::Vector3<f32>) {
        self.position = position;
    }

    pub fn position(&self) -> cgmath::Vector3<f32> {
        self.position
    }

    pub fn head_position(&self) -> cgmath::Vector3<f32> {
        self.position + cgmath::Vector3::new(0.0, self.size.y * 0.4, 0.0)
    }

    pub fn look_x(&self) -> f32 {
        self.look_x
    }

    pub fn look_y(&self) -> f32 {
        self.look_y
    }
}