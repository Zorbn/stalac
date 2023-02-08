use std::borrow::BorrowMut;

use crate::{chunk::Chunk, input::Input, gfx::instance::Instance};
use cgmath::prelude::*;

use super::{
    actor::Actor,
    ecs::{EntityManager, System},
};

pub struct EntityInstancesSystem {
    entity_instances: Vec<Instance>,
}

impl EntityInstancesSystem {
    pub fn new() -> Self {
        Self {
            entity_instances: Vec::new(),
        }
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.entity_instances
    }
}

impl System for EntityInstancesSystem {
    fn update(
        &mut self,
        ecs: &mut EntityManager,
        _entity_cache: &mut Vec<usize>,
        _chunk: &Chunk,
        _input: &mut Input,
        player: usize,
        _delta_time: f32,
    ) {
        self.entity_instances.clear();

        let player_position = ecs
            .borrow_components::<Actor>()
            .unwrap()
            .borrow_mut()
            .get(player)
            .unwrap()
            .position();
        let mut actors = ecs.borrow_components::<Actor>().unwrap();

        for actor in actors.get_all() {
            let mut instance = Instance {
                position: actor.position(),
                rotation: cgmath::Quaternion::zero(),
            };

            instance.rotate_towards(&player_position);

            self.entity_instances.push(instance);
        }
    }
}
