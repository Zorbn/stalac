use std::{borrow::BorrowMut, collections::HashSet};

use crate::{
    chunk::{Chunk, BLOCK_SIZE},
    gfx::gui::Gui,
    input::Input,
};

use super::ecs::{Ecs, System};

const GRAVITY: f32 = 30.0;
const JUMP_FORCE: f32 = 9.0;
const GROUNDED_DISTANCE: f32 = 0.1;

pub struct Actor {
    speed: f32,
    size: cgmath::Vector3<f32>,
    position: cgmath::Vector3<f32>,
    look_x: f32,
    look_y: f32,
    y_velocity: f32,
    grounded: bool,
}

impl Actor {
    pub fn new(position: cgmath::Vector3<f32>, size: cgmath::Vector3<f32>, speed: f32) -> Self {
        Self {
            position,
            size,
            speed,
            look_x: 0.0,
            look_y: 0.0,
            y_velocity: 0.0,
            grounded: false,
        }
    }

    pub fn step(
        &mut self,
        entity: usize,
        dir: cgmath::Vector3<f32>,
        speed: f32,
        chunk: &mut Chunk,
        no_clip: bool,
    ) -> bool {
        let velocity = dir * speed;
        let new_position = self.position + velocity;

        if !no_clip && chunk.get_block_collision(new_position, self.size).is_some() {
            return false;
        }

        self.update_occupied_blocks(entity, chunk, Some(new_position));
        self.position = new_position;

        true
    }

    pub fn update_occupied_blocks(
        &self,
        entity: usize,
        chunk: &mut Chunk,
        new_position: Option<cgmath::Vector3<f32>>,
    ) {
        for i in 0..4 {
            let x_offset = (i % 2) * 2 - 1;
            let z_offset = (i / 2) * 2 - 1;

            let old_block_position =
                self.get_corner_pos(self.position, x_offset, z_offset) / BLOCK_SIZE;

            chunk.remove_entity_from_block(entity, old_block_position.x, old_block_position.z);

            if let Some(new_position) = new_position {
                let new_block_position =
                    self.get_corner_pos(new_position, x_offset, z_offset) / BLOCK_SIZE;
                chunk.add_entity_to_block(entity, new_block_position.x, new_block_position.z);
            }
        }
    }

    pub fn get_nearby_entities(&mut self, chunk: &mut Chunk, nearby_entities: &mut HashSet<usize>) {
        nearby_entities.clear();

        for i in 0..4 {
            let x_offset = (i % 2) * 2 - 1;
            let z_offset = (i / 2) * 2 - 1;

            let corner_position =
                self.get_corner_pos(self.position, x_offset, z_offset) / BLOCK_SIZE;

            if let Some(entities_at_block) =
                chunk.entities_at_block(corner_position.x, corner_position.z)
            {
                nearby_entities.extend(entities_at_block);
            }
        }
    }

    pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        self.look_x += delta_x;
        self.look_y += delta_y;
        self.look_x = self.look_x.clamp(-89.0, 89.0);
    }

    pub fn teleport(&mut self, position: cgmath::Vector3<f32>) {
        self.position = position;
    }

    fn snap_to_floor(&mut self) {
        self.reset_y_velocity();
        self.position.y = self.position.y.floor() + self.size.y * 0.5;
    }

    fn reset_y_velocity(&mut self) {
        self.y_velocity = 0.0;
    }

    fn apply_gravity(&mut self, delta_time: f32) {
        self.y_velocity -= GRAVITY * delta_time;
    }

    fn get_corner_pos(
        &self,
        position: cgmath::Vector3<f32>,
        x_offset: i32,
        z_offset: i32,
    ) -> cgmath::Vector3<i32> {
        (position
            + cgmath::vec3(
                self.size.x * x_offset as f32,
                0.0,
                self.size.z * z_offset as f32,
            ))
        .cast::<i32>()
        .unwrap()
    }

    pub fn intersects(&self, position: cgmath::Vector3<f32>, size: cgmath::Vector3<f32>) -> bool {
        let min = self.position - self.size * 0.5;
        let max = self.position + self.size * 0.5;
        let other_min = position - size * 0.5;
        let other_max = position + size * 0.5;

        min.x <= other_max.x
            && max.x >= other_min.x
            && min.y <= other_max.y
            && max.y >= other_min.y
            && min.z <= other_max.z
            && max.z >= other_min.z
    }

    pub fn jump(&mut self) {
        self.y_velocity = JUMP_FORCE;
    }

    pub fn position(&self) -> cgmath::Vector3<f32> {
        self.position
    }

    pub fn size(&self) -> cgmath::Vector3<f32> {
        self.size
    }

    pub fn head_position(&self) -> cgmath::Vector3<f32> {
        self.position + cgmath::vec3(0.0, self.size.y * 0.4, 0.0)
    }

    pub fn look_x(&self) -> f32 {
        self.look_x
    }

    pub fn look_y(&self) -> f32 {
        self.look_y
    }

    pub fn y_velocity(&self) -> f32 {
        self.y_velocity
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn grounded(&self) -> bool {
        self.grounded
    }
}

pub struct ActorSystem {}

impl System for ActorSystem {
    fn update(
        &mut self,
        ecs: &mut Ecs,
        chunk: &mut Chunk,
        _input: &mut Input,
        _gui: &mut Gui,
        delta_time: f32,
    ) {
        let Ecs {
            manager,
            entity_cache,
            ..
        } = ecs;

        if !manager.get_entities_with::<Actor>(entity_cache) {
            return;
        }

        let mut actors = manager.borrow_components::<Actor>().unwrap();

        for entity in entity_cache {
            let actor = actors.borrow_mut().get_mut(*entity).unwrap();

            actor.grounded = chunk
                .get_block_collision(
                    actor.position - cgmath::vec3(0.0, GROUNDED_DISTANCE, 0.0),
                    actor.size,
                )
                .is_some();

            actor.apply_gravity(delta_time);

            if !actor.step(
                *entity,
                cgmath::Vector3::unit_y(),
                actor.y_velocity() * delta_time,
                chunk,
                false,
            ) {
                // If the player is moving towards the ground while touching it, snap to the floor
                // and prevent y_velocity from building up over time.
                if actor.y_velocity < 0.0 {
                    actor.snap_to_floor()
                }

                actor.reset_y_velocity();
            }
        }
    }
}
