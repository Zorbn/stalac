use std::borrow::BorrowMut;

use cgmath::prelude::*;
use winit::event::VirtualKeyCode;

use crate::{
    actor::Actor,
    camera::Camera,
    ecs::{EntityManager, System},
};

const MOUSE_SENSITIVITY: f32 = 0.1;

pub struct PlayerAi {}

pub struct PlayerAiSystem {}

impl System for PlayerAiSystem {
    fn update(
        &mut self,
        ecs: &mut EntityManager,
        entity_cache: &mut Vec<usize>,
        chunk: &crate::chunk::Chunk,
        input: &mut crate::input::Input,
        player: usize,
        delta_time: f32,
    ) {
        ecs.get_entities_with::<PlayerAi, Actor>(entity_cache);
        let mut actors = ecs.borrow_components::<Actor>().unwrap();

        for id in entity_cache {
            let actor = actors.borrow_mut().get(*id).unwrap();

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
}
