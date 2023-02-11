use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashSet,
};

use crate::{gfx::gui::Gui, input::Input, chunk::Chunk};

use super::{actor::Actor, ecs::{System, Ecs}, health::Health};

pub struct Fighter {
    attack_damage: i32,
    attack_cooldown: f32,
    attack_timer: f32,
}

impl Fighter {
    pub fn new(attack_damage: i32, attack_cooldown: f32) -> Self {
        Self {
            attack_damage,
            attack_cooldown,
            attack_timer: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.attack_timer -= delta_time;
    }

    pub fn get_attack(&mut self) -> i32 {
        if self.attack_timer > 0.0 {
            return 0;
        }

        self.attack_timer = self.attack_cooldown;
        self.attack_damage
    }
}

pub struct FighterSystem {
    nearby_entities: HashSet<usize>,
}

impl FighterSystem {
    pub fn new() -> Self {
        Self {
            nearby_entities: HashSet::new(),
        }
    }
}

impl System for FighterSystem {
    fn update(
        &mut self,
        ecs: &mut Ecs,
        chunk: &mut Chunk,
        _input: &mut Input,
        _gui: &mut Gui,
        delta_time: f32,
    ) {
        let Ecs { manager, entity_cache, queue } = ecs;

        manager.get_entities_with_both::<Fighter, Actor>(entity_cache);

        if entity_cache.is_empty() {
            return;
        }

        let mut actors = manager.borrow_components::<Actor>().unwrap();
        let mut fighters = manager.borrow_components::<Fighter>().unwrap();
        let mut healths = match manager.borrow_components::<Health>() {
            Some(h) => h,
            None => return,
        };

        for entity in entity_cache {
            let actor = actors.borrow_mut().get_mut(*entity).unwrap();
            let fighter = fighters.borrow_mut().get_mut(*entity).unwrap();

            fighter.update(delta_time);

            actor.get_nearby_entities(chunk, &mut self.nearby_entities);

            let position = actor.position();
            let size = actor.size();

            for nearby_entity in &self.nearby_entities {
                if nearby_entity == entity {
                    continue;
                }

                let nearby_actor = match actors.borrow().get(*nearby_entity) {
                    Some(a) => a,
                    None => continue,
                };

                if !nearby_actor.intersects(position, size) {
                    continue;
                }

                if let Some(health) = healths.get_mut(*nearby_entity) {
                    health.take_damage(fighter.get_attack());

                    if health.amount() <= 0 {
                        queue.remove_entity(*nearby_entity);
                    }
                }
            }
        }
    }
}
