use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{
    chunk::{Chunk, BLOCK_SIZE},
    direction::{dir_to_offset, index_to_dir},
};

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    priority: i32,
    position: cgmath::Vector3<i32>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(a: cgmath::Vector3<i32>, b: cgmath::Vector3<i32>) -> i32 {
    (a.x - b.x).abs() + (a.z - b.z).abs()
}

fn get_neighbors(
    position: cgmath::Vector3<i32>,
    chunk: &Chunk,
    neighbors: &mut Vec<cgmath::Vector3<i32>>,
) {
    neighbors.clear();

    for dir_i in 0..4 {
        let dir = index_to_dir(dir_i);
        let dir_offset = dir_to_offset(dir);
        let neighbor_position = cgmath::Vector3::<i32>::new(
            position.x + dir_offset.0,
            position.y + dir_offset.1,
            position.z + dir_offset.2,
        );
        if chunk.get_block(
            neighbor_position.x,
            neighbor_position.y,
            neighbor_position.z,
        ) {
            continue;
        }

        neighbors.push(neighbor_position);
    }
}

pub fn a_star_search(
    chunk: &Chunk,
    mut start: cgmath::Vector3<i32>,
    mut goal: cgmath::Vector3<i32>,
    came_from: &mut HashMap<cgmath::Vector3<i32>, cgmath::Vector3<i32>>,
) {
    start.y = goal.y;
    start /= BLOCK_SIZE as i32;
    goal /= BLOCK_SIZE as i32;

    came_from.clear();

    let mut neighbors = Vec::<cgmath::Vector3<i32>>::new();
    let mut frontier = BinaryHeap::new();
    frontier.push(Node {
        priority: 0,
        position: start,
    });

    came_from.insert(start, start);

    while !frontier.is_empty() {
        let current = frontier.pop().unwrap().position;

        if current == goal {
            break;
        }

        get_neighbors(current, chunk, &mut neighbors);
        for next in &neighbors {
            if came_from.contains_key(next) {
                continue;
            }

            let priority = heuristic(*next, goal);
            frontier.push(Node {
                priority,
                position: *next,
            });
            came_from.insert(*next, current);
        }
    }
}

pub fn reconstruct_path(
    mut start: cgmath::Vector3<i32>,
    mut goal: cgmath::Vector3<i32>,
    came_from: &mut HashMap<cgmath::Vector3<i32>, cgmath::Vector3<i32>>,
    path: &mut Vec<cgmath::Vector3<f32>>,
) {
    start.y = goal.y;
    start /= BLOCK_SIZE as i32; // TODO: Fix the need for this
    goal /= BLOCK_SIZE as i32;

    path.clear();

    let mut current = goal;

    // Return if there is no path.
    if !came_from.contains_key(&goal) {
        return;
    }

    while current != start {
        // path.push(current);
        path.push(current.cast::<f32>().unwrap() * BLOCK_SIZE + cgmath::Vector3::new(0.5, 0.5, 0.5) * BLOCK_SIZE);
        current = came_from[&current];
    }

    // path.reverse();
}
