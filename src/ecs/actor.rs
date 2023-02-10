use std::borrow::BorrowMut;

use crate::{chunk::Chunk, input::Input};

use super::ecs::{EntityManager, System};

const GRAVITY: f32 = 30.0;
const JUMP_FORCE: f32 = 9.0;

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
        let old_position = self.position;

        let velocity = dir * speed;
        self.position += velocity;

        if !no_clip
            && chunk
                .get_block_collision(self.position, self.size)
                .is_some()
        {
            self.position = old_position;
            return false;
        }

        // Since the actor has moved, it may be occupying new tiles, so the chunk must be updated.
        for i in 0..4 {
            let x_offset = (i % 2) * 2 - 1;
            let z_offset = (i / 2) * 2 - 1;

            let old_block_position = self.get_corner_pos(old_position, x_offset, z_offset);
            let new_block_position = self.get_corner_pos(self.position, x_offset, z_offset);

            chunk.remove_entity_from_block(entity, old_block_position.x, old_block_position.z);
            chunk.add_entity_to_block(entity, new_block_position.x, new_block_position.z);
        }

        true
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

    fn get_corner_pos(&self, position: cgmath::Vector3<f32>, x_offset: i32, z_offset: i32) -> cgmath::Vector3<i32> {
        (position + cgmath::Vector3::new(self.size.x * x_offset as f32, 0.0, self.size.z * z_offset as f32)).cast::<i32>().unwrap()
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
        self.position + cgmath::Vector3::new(0.0, self.size.y * 0.4, 0.0)
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
        ecs: &mut EntityManager,
        entity_cache: &mut Vec<usize>,
        chunk: &mut Chunk,
        _input: &mut Input,
        _player: usize,
        delta_time: f32,
    ) {
        ecs.get_entities_with::<Actor>(entity_cache);

        let mut actors = ecs.borrow_components::<Actor>().unwrap();

        for entity in entity_cache {
            let actor = actors.borrow_mut().get(*entity).unwrap();

            actor.grounded = chunk
                .get_block_collision(
                    actor.position - cgmath::Vector3::new(0.0, 0.01, 0.0),
                    actor.size,
                )
                .is_some();

            // If the player is moving towards the ground while touching it, snap to the floor
            // and prevent y_velocity from building up over time.
            if actor.grounded && actor.y_velocity < 0.0 {
                actor.snap_to_floor()
            }

            actor.apply_gravity(delta_time);

            if !actor.step(
                *entity,
                cgmath::Vector3::unit_y(),
                actor.y_velocity() * delta_time,
                chunk,
                false,
            ) {
                actor.reset_y_velocity();
            }
        }
    }
}
