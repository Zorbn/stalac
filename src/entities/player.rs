use std::{borrow::BorrowMut, collections::HashSet};

use cgmath::prelude::*;
use winit::event::{MouseButton, VirtualKeyCode};

use crate::{
    chunk::{Chunk, BLOCK_SIZE_F},
    gfx::{camera::Camera, gui::Gui},
    input::Input,
};

use super::{
    actor::Actor,
    ecs::{Ecs, System},
};

const MOUSE_SENSITIVITY: f32 = 0.1;

pub struct Player {}

pub struct PlayerMovementSystem {
    hit_entities: HashSet<usize>,
}

impl PlayerMovementSystem {
    pub fn new() -> Self {
        Self { hit_entities: HashSet::new() }
    }
}

impl System for PlayerMovementSystem {
    fn update(
        &mut self,
        ecs: &mut Ecs,
        chunk: &mut Chunk,
        input: &mut Input,
        _gui: &mut Gui,
        delta_time: f32,
    ) {
        let Ecs {
            manager,
            entity_cache,
            ..
        } = ecs;

        if !manager.get_entities_with_both::<Player, Actor>(entity_cache) {
            return;
        }

        let mut actors = manager.borrow_components::<Actor>().unwrap();

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

            if input.was_mouse_button_pressed(MouseButton::Left) {
                let start = actor.position() / BLOCK_SIZE_F;
                let dir = Camera::get_direction_vec(actor.look_y());

                let hit = chunk.raycast(start, dir, 10.0, Some(&mut self.hit_entities));

                if let Some(hit) = hit {
                    chunk.set_block(false, hit.position.x, hit.position.y, hit.position.z);
                }

                for hit_entity in self.hit_entities.iter() {
                    if *entity == *hit_entity {
                        continue;
                    }
                    println!("hit: {}", *hit_entity);
                }
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
                cgmath::vec3(dir.x, 0.0, 0.0),
                actor.speed() * delta_time,
                chunk,
                no_clip,
            );
            actor.step(
                *entity,
                cgmath::vec3(0.0, 0.0, dir.z),
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
