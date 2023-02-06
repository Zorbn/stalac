use cgmath::prelude::*;
use std::collections::HashMap;

use crate::{
    a_star::{a_star_search, reconstruct_path},
    ai::Ai,
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

impl Ai for ChaseAi {
    // TODO: When the ai is at the closest tile, run directly towards the player (no pathing)
    // until they are touching (within a constant distance, maybe 1m)
    fn update(
        &mut self,
        actor: &mut crate::actor::Actor,
        _input: &mut crate::input::Input,
        player_position: cgmath::Vector3<f32>,
        chunk: &crate::chunk::Chunk,
        delta_time: f32,
    ) {
        self.repath_timer += delta_time;

        let position = actor.position();

        if self.repath_timer > REPATH_TIME {
            self.repath_timer = 0.0;

            let mut came_from = HashMap::<cgmath::Vector3<i32>, cgmath::Vector3<i32>>::new();
            let start = position.cast::<i32>().unwrap();
            let goal = player_position.cast::<i32>().unwrap();
            a_star_search(chunk, start, goal, &mut came_from);
            reconstruct_path(start, goal, &mut came_from, &mut self.path);
            self.next = self.path.pop();
        }

        if let Some(next) = self.next {
            let next_f = next.cast::<f32>().unwrap();

            let x_dist = next_f.x - position.x;
            let z_dist = next_f.z - position.z;
            if (x_dist * x_dist + z_dist * z_dist).sqrt() < 0.5 {
                self.next = None;
                return;
            }

            let dir =
                cgmath::Vector3::new(next_f.x - position.x, 0.0, next_f.z - position.z).normalize();
            actor.step(dir, 4.0 * delta_time, chunk, true);
        } else {
            self.next = self.path.pop();
        }
    }
}
