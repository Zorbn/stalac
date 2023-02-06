use cgmath::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::{chunk::Chunk, entity::Entity, input::Input, instance::Instance};

pub struct Entities {
    data: HashMap<u32, Entity>,
    player_ids: HashSet<u32>,
    next_id: u32,
    instances: Vec<Instance>,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            player_ids: HashSet::new(),
            next_id: 0,
            instances: Vec::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, is_player: bool) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.data.insert(id, entity);

        if is_player {
            self.player_ids.insert(id);
        }

        id
    }

    pub fn remove(&mut self, id: u32) {
        self.data.remove(&id);
        self.player_ids.remove(&id);
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Entity> {
        self.data.get_mut(&id)
    }

    pub fn get(&self, id: u32) -> Option<&Entity> {
        self.data.get(&id)
    }

    pub fn is_player(&self, id: u32) -> bool {
        self.player_ids.contains(&id)
    }

    pub fn contains(&self, id: u32) -> bool {
        self.data.contains_key(&id)
    }

    pub fn update(
        &mut self,
        input: &mut Input,
        player_position: cgmath::Vector3<f32>,
        chunk: &Chunk,
        delta_time: f32,
    ) {
        self.instances.clear();

        for (id, entity) in &mut self.data {
            entity.update(input, player_position, chunk, delta_time);

            if self.player_ids.contains(id) {
                continue;
            }

            let mut instance = Instance {
                position: entity.actor.position(),
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
