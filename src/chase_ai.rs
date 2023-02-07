use cgmath::prelude::*;
use std::collections::HashMap;

use crate::{
    a_star::{a_star_search, reconstruct_path}, entities::Entities,
};

const REPATH_TIME: f32 = 1.0;

pub struct ChaseAi {
    pub repath_timer: f32,
    pub path: Vec<cgmath::Vector3<f32>>,
    pub next: Option<cgmath::Vector3<f32>>,
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

impl ChaseAi {
    // TODO: When the ai is at the closest tile, run directly towards the player (no pathing)
    // until they are touching (within a constant distance, maybe 1m)
    pub fn update(
        self_id: u32,
        _input: &mut crate::input::Input,
        entities: &mut Entities,
        player_id: u32,
        chunk: &crate::chunk::Chunk,
        delta_time: f32,
    ) {
        let player_position = entities.actor.get_mut(&player_id).unwrap().position();
        let ai = entities.chase_ai.get_mut(&self_id).unwrap();
        let actor = entities.actor.get_mut(&self_id).unwrap();

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

            let dir =
                cgmath::Vector3::new(next_f.x - position.x, 0.0, next_f.z - position.z).normalize();
            actor.step(dir, 4.0 * delta_time, chunk, true);
        } else {
            ai.next = ai.path.pop();
        }
    }
}
