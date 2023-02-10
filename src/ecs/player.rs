use std::borrow::{Borrow, BorrowMut};

use cgmath::prelude::*;
use winit::event::VirtualKeyCode;

use crate::{chunk::Chunk, gfx::camera::Camera, input::Input};

use super::{
    actor::Actor,
    ecs::{EntityManager, System},
};

const MOUSE_SENSITIVITY: f32 = 0.1;

pub struct Player {}

pub struct PlayerMovementSystem {}

impl System for PlayerMovementSystem {
    fn update(
        &mut self,
        ecs: &mut EntityManager,
        entity_cache: &mut Vec<usize>,
        chunk: &mut Chunk,
        input: &mut Input,
        _player: usize,
        delta_time: f32,
    ) {
        ecs.get_entities_with_both::<Player, Actor>(entity_cache);

        if entity_cache.len() == 0 {
            return;
        }

        let mut actors = ecs.borrow_components::<Actor>().unwrap();

        for entity in entity_cache {
            let actor = actors.borrow_mut().get_mut(*entity).unwrap();

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
                *entity,
                cgmath::Vector3::new(dir.x, 0.0, 0.0),
                actor.speed() * delta_time,
                chunk,
                no_clip,
            );
            actor.step(
                *entity,
                cgmath::Vector3::new(0.0, 0.0, dir.z),
                actor.speed() * delta_time,
                chunk,
                no_clip,
            );

            actor.rotate(
                input.mouse_delta_y() * MOUSE_SENSITIVITY,
                -input.mouse_delta_x() * MOUSE_SENSITIVITY,
            );

            actor.update_nearby_entities(chunk);

            // TODO: Create an attack component which has a get_next_attack_damage function, that returns
            // for example, 0 if the attack hasn't charged, and 10 if it has, add all get_next_attack_damage's
            // to a counter, then outside the loop drop the immutable borrow to the player, get a new mutable one,
            // and apply all of the damage at once.
            // drop(actor);

            // let actor = actors.borrow().get(*entity).unwrap();

            // for nearby_entity in actor.nearby_entities() {
            //     if nearby_entity != entity {
            //         let other_actor = actors.borrow().get(*nearby_entity).unwrap();
            //         println!(
            //             "{}",
            //             actor.intersects(other_actor.position(), other_actor.size())
            //         );
            //     }
            // }
        }
    }
}
