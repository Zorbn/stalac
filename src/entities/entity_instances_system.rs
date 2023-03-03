use std::borrow::{Borrow, BorrowMut};

use crate::{
    chunk::Chunk,
    gfx::{camera::get_look_direction, gui::Gui, instance::Instance},
    input::Input,
};
use cgmath::prelude::*;

use super::{
    actor::Actor,
    display::Display,
    ecs::{Ecs, System},
    player::Player,
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
        ecs: &mut Ecs,
        _chunk: &mut Chunk,
        _input: &mut Input,
        _gui: &mut Gui,
        _delta_time: f32,
    ) {
        let Ecs {
            manager,
            entity_cache,
            ..
        } = ecs;

        manager.get_entities_with_both::<Player, Actor>(entity_cache);

        let player = match entity_cache.first() {
            Some(p) => *p,
            None => return,
        };

        self.entity_instances.clear();

        if !manager.get_entities_with_both::<Display, Actor>(entity_cache) {
            return;
        }

        let mut displays = manager.borrow_components::<Display>().unwrap();
        let mut actors = manager.borrow_components::<Actor>().unwrap();

        let player_actor = actors.borrow().get(player).unwrap();

        let player_look_direction =
            get_look_direction(player_actor.look_x(), player_actor.look_y());

        for entity in entity_cache {
            let display = displays.borrow_mut().get(*entity).unwrap();
            let actor = actors.borrow_mut().get(*entity).unwrap();

            let mut instance = Instance {
                position: actor.position(),
                rotation: cgmath::Quaternion::zero(),
                tex_index: display.tex_index(),
            };

            instance.billboard(player_look_direction);

            self.entity_instances.push(instance);
        }
    }
}
