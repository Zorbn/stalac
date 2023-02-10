use cgmath::prelude::*;
use std::{borrow::BorrowMut, collections::HashMap};

use crate::{
    a_star::{a_star_search, reconstruct_path},
    chunk::Chunk,
    input::Input,
};

use super::{
    actor::Actor,
    ecs::{EntityManager, System},
};

const REPATH_TIME: f32 = 1.0;

pub struct ChaseAi {
    repath_timer: f32,
    path: Vec<cgmath::Vector3<f32>>,
    next: Option<cgmath::Vector3<f32>>,
}

impl ChaseAi {
    pub fn new() -> Self {
        Self {
            repath_timer: 0.0,
            path: Vec::new(),
            next: None,
        }
    }
}

pub struct ChaseAiSystem {}

impl System for ChaseAiSystem {
    // TODO: When the ai is at the closest tile, run directly towards the player (no pathing)
    // until they are touching (within a constant distance, maybe 1m)
    fn update(
        &mut self,
        ecs: &mut EntityManager,
        entity_cache: &mut Vec<usize>,
        chunk: &mut Chunk,
        _input: &mut Input,
        player: usize,
        delta_time: f32,
    ) {
        let player_position = ecs
            .borrow_components::<Actor>()
            .unwrap()
            .borrow_mut()
            .get(player)
            .unwrap()
            .position();

        ecs.get_entities_with_both::<ChaseAi, Actor>(entity_cache);

        if entity_cache.len() == 0 {
            return;
        }

        let mut ais = ecs.borrow_components::<ChaseAi>().unwrap();
        let mut actors = ecs.borrow_components::<Actor>().unwrap();

        for entity in entity_cache {
            let ai = ais.borrow_mut().get_mut(*entity).unwrap();
            let actor = actors.borrow_mut().get_mut(*entity).unwrap();

            ai.repath_timer += delta_time;

            let position = actor.position();

            if ai.repath_timer > REPATH_TIME {
                ai.repath_timer = 0.0;

                let mut came_from = HashMap::<cgmath::Vector3<i32>, cgmath::Vector3<i32>>::new();
                let start = position.cast::<i32>().unwrap();
                let goal = player_position.cast::<i32>().unwrap();
                a_star_search(chunk, start, goal, &mut came_from);
                reconstruct_path(start, goal, &mut came_from, &mut ai.path);
                ai.next = ai.path.pop();
            }

            if let Some(next) = ai.next {
                let next_f = next.cast::<f32>().unwrap();

                let x_dist = next_f.x - position.x;
                let z_dist = next_f.z - position.z;
                if (x_dist * x_dist + z_dist * z_dist).sqrt() < 0.5 {
                    ai.next = None;
                    return;
                }

                let dir = cgmath::Vector3::new(next_f.x - position.x, 0.0, next_f.z - position.z)
                    .normalize();
                actor.step(*entity, dir, 4.0 * delta_time, chunk, true);
            } else {
                ai.next = ai.path.pop();
            }
        }
    }
}
