use cgmath::prelude::*;
use winit::event::VirtualKeyCode;

use crate::{actor::Actor, ai::Ai, camera::Camera, chunk::Chunk, input::Input};

const MOUSE_SENSITIVITY: f32 = 0.1;

pub struct PlayerAi {}

impl Ai for PlayerAi {
    fn update(&mut self, actor: &mut Actor, input: &mut Input, _player_position: cgmath::Vector3<f32>, chunk: &Chunk, delta_time: f32) {
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

        if actor.grounded() && input.is_key_held(VirtualKeyCode::Space) {
            actor.jump();
        }

        let no_clip = input.is_key_held(VirtualKeyCode::V);

        let forward = Camera::get_direction_vec(actor.look_y());
        let right = Camera::get_direction_vec(actor.look_y() + 90.0);
        let mut dir = dir_z * forward + dir_x * right;

        if dir.magnitude() != 0.0 {
            dir = dir.normalize();
        }

        actor.step(
            cgmath::Vector3::new(dir.x, 0.0, 0.0),
            actor.speed() * delta_time,
            chunk,
            no_clip,
        );
        actor.step(
            cgmath::Vector3::new(0.0, 0.0, dir.z),
            actor.speed() * delta_time,
            chunk,
            no_clip,
        );

        actor.rotate(
            input.mouse_delta_y() * MOUSE_SENSITIVITY,
            -input.mouse_delta_x() * MOUSE_SENSITIVITY,
        );
    }
}
