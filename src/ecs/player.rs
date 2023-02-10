use std::{borrow::BorrowMut, collections::HashSet};

use cgmath::prelude::*;
use winit::event::VirtualKeyCode;

use crate::{chunk::Chunk, gfx::camera::Camera, input::Input};

use super::{
    actor::Actor,
    ecs::{EntityManager, System}, fighter::Fighter,
};

const MOUSE_SENSITIVITY: f32 = 0.1;

pub struct Player {}

pub struct PlayerMovementSystem {
    nearby_entities: HashSet<usize>,
}

impl PlayerMovementSystem {
    pub fn new() -> Self {
        Self { nearby_entities: HashSet::new() }
    }
}

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
        let mut fighters = ecs.borrow_components::<Fighter>();

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

            actor.get_nearby_entities(chunk, &mut self.nearby_entities);

            let position = actor.position();
            let size = actor.size();

            drop(actor);

            // TODO: Fighters should have their own system to attack the player with.
            let fighters = match &mut fighters {
                Some(f) => f,
                None => continue,
            };

            let mut damage_accumulator = 0;

            for nearby_entity in &self.nearby_entities {
                if nearby_entity == entity {
                    continue;
                }

                let nearby_actor = actors.borrow_mut().get_mut(*nearby_entity).unwrap();
                if !nearby_actor.intersects(position, size) {
                    continue;
                }

                if let Some(fighter) = fighters.borrow_mut().get_mut(*nearby_entity) {
                    damage_accumulator += fighter.get_attack();
                }
            }

            println!("{}", damage_accumulator);
        }
    }
}
