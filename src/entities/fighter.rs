use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefMut,
    collections::HashSet,
};

use winit::event::MouseButton;

use crate::{
    chunk::{Chunk, BLOCK_SIZE_F},
    gfx::{camera::Camera, gui::Gui},
    input::Input,
    ray::Ray,
};

use super::{
    actor::Actor,
    ecs::{ComponentStore, Ecs, System},
    health::Health,
    player::Player,
};

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

    fn get_target(
        &mut self,
        entity: usize,
        chunk: &mut Chunk,
        input: &mut Input,
        players: &Option<RefMut<ComponentStore<Player>>>,
        actors: &mut RefMut<ComponentStore<Actor>>,
        healths: &RefMut<ComponentStore<Health>>,
    ) -> Option<usize> {
        if let Some(ref players) = players {
            if players.borrow().has(entity) {
                self.get_target_raycast(entity, chunk, input, actors, healths)
            } else {
                self.get_target_proximity(entity, chunk, actors, healths)
            }
        } else {
            self.get_target_proximity(entity, chunk, actors, healths)
        }
    }

    fn get_target_proximity(
        &mut self,
        entity: usize,
        chunk: &mut Chunk,
        actors: &mut RefMut<ComponentStore<Actor>>,
        healths: &RefMut<ComponentStore<Health>>,
    ) -> Option<usize> {
        let actor = actors.borrow_mut().get_mut(entity).unwrap();

        actor.get_nearby_entities(chunk, &mut self.nearby_entities);

        let position = actor.position();
        let size = actor.size();

        for nearby_entity in &self.nearby_entities {
            if *nearby_entity == entity {
                continue;
            }

            let nearby_actor = match actors.borrow_mut().get(*nearby_entity) {
                Some(a) => a,
                None => continue,
            };

            if !nearby_actor.intersects(position, size) {
                continue;
            }

            if healths.has(*nearby_entity) {
                return Some(*nearby_entity);
            }
        }

        None
    }

    fn get_target_raycast(
        &mut self,
        entity: usize,
        chunk: &mut Chunk,
        input: &mut Input,
        actors: &mut RefMut<ComponentStore<Actor>>,
        healths: &RefMut<ComponentStore<Health>>,
    ) -> Option<usize> {
        let actor = actors.borrow_mut().get_mut(entity).unwrap();

        let position = actor.position();
        let look_y = actor.look_y();

        if input.was_mouse_button_pressed(MouseButton::Left) {
            let start = position / BLOCK_SIZE_F;
            let dir = Camera::get_direction_vec(look_y);

            chunk.raycast(start, dir, 10.0, Some(&mut self.nearby_entities));

            for hit_entity in &self.nearby_entities {
                if entity == *hit_entity {
                    continue;
                }

                if !healths.has(*hit_entity) {
                    continue;
                }

                if let Some(hit_actor) = actors.get(*hit_entity) {
                    let ray = Ray { position, dir };

                    if ray.intersects(hit_actor.position(), hit_actor.size()) {
                        return Some(*hit_entity);
                    }
                }
            }
        }

        None
    }
}

impl System for FighterSystem {
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

        if !manager.get_entities_with_both::<Fighter, Actor>(entity_cache) {
            return;
        }

        let mut actors = manager.borrow_components::<Actor>().unwrap();
        let mut fighters = manager.borrow_components::<Fighter>().unwrap();
        let players = manager.borrow_components::<Player>();
        let mut healths = match manager.borrow_components::<Health>() {
            Some(h) => h,
            None => return,
        };

        for entity in entity_cache {
            let fighter = fighters.borrow_mut().get_mut(*entity).unwrap();

            fighter.update(delta_time);

            // Find a target actor with health that this entity can hit, AI characters and players
            // use different methods to find a target.
            let target =
                match self.get_target(*entity, chunk, input, &players, &mut actors, &healths) {
                    Some(t) => t,
                    None => continue,
                };

            if let Some(health) = healths.get_mut(target) {
                println!("{} hit {}", *entity, target);
                health.take_damage(fighter.get_attack());
            }
        }
    }
}
