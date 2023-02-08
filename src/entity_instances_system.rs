use std::borrow::BorrowMut;

use crate::{actor::Actor, ecs::System, instance::Instance};
use cgmath::prelude::*;

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
        ecs: &mut crate::ecs::EntityManager,
        entity_cache: &mut Vec<usize>,
        chunk: &crate::chunk::Chunk,
        input: &mut crate::input::Input,
        player: usize,
        delta_time: f32,
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
