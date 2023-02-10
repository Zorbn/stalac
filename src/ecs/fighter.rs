use std::{borrow::{Borrow, BorrowMut}, collections::HashSet};

use crate::ecs::actor::Actor;

use super::{ecs::System, health::Health};

pub struct Fighter {
    attack_cooldown: f32,
}

impl Fighter {
    pub fn new() -> Self {
        Self {
            attack_cooldown: 1.0,
        }
    }

    pub fn get_attack(&mut self) -> i32 {
        self.attack_cooldown = 0.0;
        10
    }
}

pub struct FighterSystem {
    nearby_entities: HashSet<usize>,
}

impl FighterSystem {
    pub fn new() -> Self {
        Self { nearby_entities: HashSet::new() }
    }
}

impl System for FighterSystem {
    fn update(
        &mut self,
        ecs: &mut super::ecs::EntityManager,
        entity_cache: &mut Vec<usize>,
        chunk: &mut crate::chunk::Chunk,
        _input: &mut crate::input::Input,
        _player: usize,
        _delta_time: f32,
    ) {

        ecs.get_entities_with_both::<Fighter, Actor>(entity_cache);

        let mut actors = ecs.borrow_components::<Actor>().unwrap();
        let mut fighters = match ecs.borrow_components::<Fighter>() {
            Some(f) => f,
            None => return,
        };
        let mut healths = match ecs.borrow_components::<Health>() {
            Some(h) => h,
            None => return,
        };

        for entity in entity_cache {
            let actor = actors.borrow_mut().get_mut(*entity).unwrap();
            let fighter = fighters.borrow_mut().get_mut(*entity).unwrap();

            actor.get_nearby_entities(chunk, &mut self.nearby_entities);

            let position = actor.position();
            let size = actor.size();

            drop(actor);

            for nearby_entity in &self.nearby_entities {
                if nearby_entity == entity {
                    continue;
                }

                let nearby_actor = actors.borrow().get(*nearby_entity).unwrap();
                if !nearby_actor.intersects(position, size) {
                    continue;
                }

                if let Some(health) = healths.get_mut(*nearby_entity) {
                    health.take_damage(fighter.get_attack());
                }
            }
        }
    }
}