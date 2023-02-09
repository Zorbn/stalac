use std::borrow::BorrowMut;

use crate::{chunk::Chunk, input::Input, gfx::instance::Instance};
use cgmath::prelude::*;

use super::{
    actor::Actor,
    ecs::{EntityManager, System}, display::Display,
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
        entity_cache: &mut Vec<usize>,
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

        ecs.get_entities_with::<Display, Actor>(entity_cache);

        if entity_cache.len() == 0 {
            return;
        }

        let mut displays = ecs.borrow_components::<Display>().unwrap();
        let mut actors = ecs.borrow_components::<Actor>().unwrap();

        for id in entity_cache {
            let display = displays.borrow_mut().get(*id).unwrap();
            let actor = actors.borrow_mut().get(*id).unwrap();

            let mut instance = Instance {
                position: actor.position(),
                rotation: cgmath::Quaternion::zero(),
                tex_index: display.tex_index(),
            };

            instance.rotate_towards(&player_position);

            self.entity_instances.push(instance);
        }
    }
}
