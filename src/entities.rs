use cgmath::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::{chunk::Chunk, input::Input, instance::Instance, actor::Actor, chase_ai::ChaseAi, player_ai::{PlayerAi}};

pub struct Entities {
    pub actor: HashMap<u32, Actor>,
    pub chase_ai: HashMap<u32, ChaseAi>,
    pub player_ai: HashMap<u32, PlayerAi>,

    instances: Vec<Instance>,
    next_id: u32,
    keys: Vec<u32>,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            actor: HashMap::new(),
            chase_ai: HashMap::new(),
            player_ai: HashMap::new(),
            next_id: 0,
            instances: Vec::new(),
            keys: Vec::new(),
        }
    }

    pub fn insert(&mut self, actor: Actor,
        chase_ai: Option<ChaseAi>,
        player_ai: Option<PlayerAi>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.actor.insert(id, actor);

        if let Some(chase_ai) = chase_ai {
            self.chase_ai.insert(id, chase_ai);
        }

        if let Some(player_ai) = player_ai {
            self.player_ai.insert(id, player_ai);
        }

        id
    }

    pub fn remove(&mut self, id: u32) {
        self.actor.remove(&id);
        self.chase_ai.remove(&id);
        self.player_ai.remove(&id);
    }

    pub fn contains(&self, id: u32) -> bool {
        self.actor.contains_key(&id)
    }

    pub fn update(
        &mut self,
        entity_key_cache: &mut Vec<u32>,
        input: &mut Input,
        player_id: u32,
        chunk: &Chunk,
        delta_time: f32,
    ) {
        self.instances.clear();

        entity_key_cache.clear();

        for id in self.actor.keys() {
            entity_key_cache.push(*id);
        }

        let player_position = self.actor.get(&player_id).expect("Player not found!").position();

        for id in entity_key_cache {
            // Entity::update(*id, input, self, player_id, chunk, delta_time);
            Actor::update(*id, self, input, chunk, delta_time);

            if self.chase_ai.contains_key(id) {
                ChaseAi::update(*id, input, self, player_id, chunk, delta_time);
            }

            if self.player_ai.contains_key(id) {
                PlayerAi::update(*id, input, self, player_id, chunk, delta_time);
                continue;
            }

            let mut instance = Instance {
                position: self.actor.get(id).unwrap().position(),
                rotation: cgmath::Quaternion::zero(),
            };

            instance.rotate_towards(&player_position);

            self.instances.push(instance);
        }
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }
}
