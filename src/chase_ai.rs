use std::collections::HashMap;
use cgmath::prelude::*;

use crate::{ai::Ai, a_star::{a_star_search, reconstruct_path}};

const REPATH_TIME: f32 = 1.0;

pub struct ChaseAi {
    repath_timer: f32,
    path: Vec<cgmath::Vector3<f32>>,
}

impl ChaseAi {
    pub fn new() -> Self {
        Self {
            repath_timer: 0.0,
            path: Vec::new(),
        }
    }
}

impl Ai for ChaseAi {
    fn update(&mut self, actor: &mut crate::actor::Actor, _input: &mut crate::input::Input, player_position: cgmath::Vector3<f32>, chunk: &crate::chunk::Chunk, delta_time: f32) {
        self.repath_timer += delta_time;

        let position = actor.position();

        if self.repath_timer > REPATH_TIME {
            self.repath_timer = 0.0;

            let mut came_from = HashMap::<cgmath::Vector3<i32>, cgmath::Vector3<i32>>::new();
            let start = position.cast::<i32>().unwrap();
            let goal = player_position.cast::<i32>().unwrap();
            a_star_search(chunk, start, goal, &mut came_from);
            reconstruct_path(start, goal, &mut came_from, &mut self.path);
        }

        if let Some(next) = self.path.last() {
            let next_f = next.cast::<f32>().unwrap();

            if next_f.distance(position) < 1.0 {
                self.path.pop();
                return;
            }

            let dir = cgmath::Vector3::new(next_f.x - position.x, 0.0, next_f.z - position.z).normalize();
            actor.step(dir, 4.0 * delta_time, chunk, true);
        }
    }
}