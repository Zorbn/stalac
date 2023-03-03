use std::collections::HashSet;

use crate::direction::{dir_outward_component, dir_to_offset, index_to_dir, Direction};
use crate::gfx::cube_mesh::{CUBE_INDICES, CUBE_VERTICES};
use crate::gfx::instance::Instance;
use crate::gfx::model::Model;
use crate::gfx::vertex::Vertex;
use crate::math::round_vec_to_i32;
use crate::rng::Rng;
use cgmath::prelude::*;

pub const BLOCK_SIZE: i32 = 3;
pub const BLOCK_SIZE_F: f32 = BLOCK_SIZE as f32;
const CHUNK_SIZE: usize = 32;
const CHUNK_HEIGHT: usize = 8;
const CHUNK_LEN: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;
const INV_BLOCK_SIZE: f32 = 1.0 / BLOCK_SIZE as f32;

pub struct RaycastHit {
    pub distance: f32,
    pub position: cgmath::Vector3<i32>,
    pub last_position: cgmath::Vector3<i32>,
}

struct VertexNeighbors {
    side1: bool,
    side2: bool,
    corner: bool,
}

pub struct Chunk {
    model: Option<Model>,
    blocks: [bool; CHUNK_LEN],
    entities_on_blocks: Vec<HashSet<usize>>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    ao_buffer: [i32; 4],
    is_dirty: bool,
}

impl Chunk {
    pub fn new() -> Self {
        let mut entities_on_blocks = Vec::new();
        entities_on_blocks.reserve(CHUNK_LEN);
        for _ in 0..CHUNK_LEN {
            entities_on_blocks.push(HashSet::new());
        }

        Self {
            blocks: [false; CHUNK_LEN],
            entities_on_blocks,
            model: None,
            vertices: Vec::new(),
            indices: Vec::new(),
            ao_buffer: [0; 4],
            is_dirty: false,
        }
    }

    pub fn generate_blocks(&mut self, rng: &mut Rng) {
        // for z in 0..CHUNK_SIZE {
        //     for y in 0..CHUNK_HEIGHT {
        //         for x in 0..CHUNK_SIZE {
        //             if rng.range(100) < 50 {
        //                 continue;
        //             }

        //             self.set_block(true, x as i32, y as i32, z as i32);
        //         }
        //     }
        // }

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                self.set_block(true, x as i32, 0, z as i32);
            }
        }

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if rng.range(100) > 10 {
                    continue;
                }

                self.set_block(true, x as i32, 1, z as i32);
            }
        }
    }

    pub fn update_mesh(&mut self, device: &wgpu::Device) {
        if self.is_dirty {
            self.generate_mesh(device);
        }
    }

    pub fn generate_mesh(&mut self, device: &wgpu::Device) {
        self.is_dirty = false;

        self.vertices.clear();
        self.indices.clear();

        for z in 0..CHUNK_SIZE {
            let iz = z as i32;
            for y in 0..CHUNK_HEIGHT {
                let iy = y as i32;
                for x in 0..CHUNK_SIZE {
                    let ix = x as i32;

                    let block = self.get_block(ix, iy, iz);

                    if !block {
                        continue;
                    }

                    for dir_i in 0..6 {
                        let dir = index_to_dir(dir_i);
                        let dir_offset = dir_to_offset(dir);
                        if self.get_block(
                            ix + dir_offset[0],
                            iy + dir_offset[1],
                            iz + dir_offset[2],
                        ) {
                            continue;
                        }

                        let vert_count = self.vertices.len() as u32;

                        for vert_i in 0..4 {
                            let mut vert = CUBE_VERTICES[dir_i][vert_i];
                            vert.position[0] = (vert.position[0] + ix as f32) * BLOCK_SIZE_F;
                            vert.position[1] = (vert.position[1] + iy as f32) * BLOCK_SIZE_F;
                            vert.position[2] = (vert.position[2] + iz as f32) * BLOCK_SIZE_F;

                            let neighbors = self.check_vertex_neighbors(
                                [ix, iy, iz],
                                [
                                    CUBE_VERTICES[dir_i][vert_i].position[0] as i32,
                                    CUBE_VERTICES[dir_i][vert_i].position[1] as i32,
                                    CUBE_VERTICES[dir_i][vert_i].position[2] as i32,
                                ],
                                dir,
                            );
                            let ao = Chunk::calculate_ao_level(neighbors);
                            self.ao_buffer[vert_i] = ao;
                            let ao_light_value = ao as f32 * 0.33;
                            vert.color[0] = ao_light_value;
                            vert.color[1] = ao_light_value;
                            vert.color[2] = ao_light_value;

                            self.vertices.push(vert);
                        }

                        for ind_i in 0..6 {
                            self.indices.push(CUBE_INDICES[dir_i][ind_i] + vert_count);
                        }

                        self.orient_last_face();
                    }
                }
            }
        }

        self.model = Some(Model::new(device, &self.vertices, &self.indices));

        if let Some(model) = &mut self.model {
            model.update_instances(
                device,
                &vec![Instance {
                    position: cgmath::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    rotation: cgmath::Quaternion::zero(),
                    tex_index: 0,
                }],
            );
        }
    }

    pub fn set_block(&mut self, solid: bool, x: i32, y: i32, z: i32) {
        let i_chunk_size = CHUNK_SIZE as i32;
        let i_chunk_height = CHUNK_HEIGHT as i32;
        if x < 0 || x >= i_chunk_size || y < 0 || y >= i_chunk_height || z < 0 || z >= i_chunk_size
        {
            return;
        }

        let ux = x as usize;
        let uy = y as usize;
        let uz = z as usize;

        self.blocks[ux + uy * CHUNK_SIZE + uz * CHUNK_SIZE * CHUNK_HEIGHT] = solid;
        self.is_dirty = true;
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> bool {
        let i_chunk_size = CHUNK_SIZE as i32;
        let i_chunk_height = CHUNK_HEIGHT as i32;
        if x < 0 || x >= i_chunk_size || y < 0 || y >= i_chunk_height || z < 0 || z >= i_chunk_size
        {
            return true;
        }

        let ux = x as usize;
        let uy = y as usize;
        let uz = z as usize;

        self.blocks[ux + uy * CHUNK_SIZE + uz * CHUNK_SIZE * CHUNK_HEIGHT]
    }

    pub fn get_block_collision(
        &self,
        mut position: cgmath::Vector3<f32>,
        mut size: cgmath::Vector3<f32>,
    ) -> Option<cgmath::Vector3<i32>> {
        position *= INV_BLOCK_SIZE;
        size *= INV_BLOCK_SIZE;

        let start = position - size * 0.5;

        let steps = round_vec_to_i32(size) + cgmath::vec3(1, 1, 1);
        let mut interp = cgmath::vec3(0.0, 0.0, 0.0);

        for x in 0..=steps.x {
            interp.x = start.x + x as f32 / steps.x as f32 * size.x;
            for y in 0..=steps.y {
                interp.y = start.y + y as f32 / steps.y as f32 * size.y;
                for z in 0..=steps.z {
                    interp.z = start.z + z as f32 / steps.z as f32 * size.z;

                    let interp_block = round_vec_to_i32(interp);

                    if self.get_block(interp_block.x, interp_block.y, interp_block.z) {
                        return Some(interp_block);
                    }
                }
            }
        }

        None
    }

    pub fn raycast(
        &self,
        start: cgmath::Vector3<f32>,
        dir: cgmath::Vector3<f32>,
        range: f32,
        mut hit_entities: Option<&mut HashSet<usize>>,
    ) -> Option<RaycastHit> {
        if let Some(ref mut hit_entities) = hit_entities {
            hit_entities.clear();
        }

        let tile_dir = dir.map(|n| n.signum()).cast::<i32>().unwrap();
        let step = (1.0 / dir).map(|n| n.abs());
        let mut initial_step = cgmath::Vector3::zero();

        initial_step.x = if dir.x > 0.0 {
            start.x.ceil() - start.x
        } else {
            start.x - start.x.floor()
        } * step.x;

        initial_step.y = if dir.y > 0.0 {
            start.y.ceil() - start.y
        } else {
            start.y - start.y.floor()
        } * step.y;

        initial_step.z = if dir.z > 0.0 {
            start.z.ceil() - start.z
        } else {
            start.z - start.z.floor()
        } * step.z;

        let mut dist_to_next = initial_step;
        let mut block_pos = start.map(|n| n.floor()).cast::<i32>().unwrap();
        let mut last_pos = block_pos;
        let mut last_dist_to_next: f32 = 0.0;

        let mut hit_block = self.get_block(block_pos.x, block_pos.y, block_pos.z);
        while !hit_block && last_dist_to_next < range {
            last_pos = block_pos;

            if let Some(ref mut hit_entities) = hit_entities {
                if let Some(entities_at_block) = self.entities_at_block(block_pos.x, block_pos.z) {
                    for hit_entity in entities_at_block {
                        hit_entities.insert(*hit_entity);
                    }
                }
            }

            if dist_to_next.x < dist_to_next.y && dist_to_next.x < dist_to_next.z {
                last_dist_to_next = dist_to_next.x;
                dist_to_next.x += step.x;
                block_pos.x += tile_dir.x;
            } else if dist_to_next.y < dist_to_next.x && dist_to_next.y < dist_to_next.z {
                last_dist_to_next = dist_to_next.y;
                dist_to_next.y += step.y;
                block_pos.y += tile_dir.y;
            } else {
                last_dist_to_next = dist_to_next.z;
                dist_to_next.z += step.z;
                block_pos.z += tile_dir.z;
            }

            hit_block = self.get_block(block_pos.x, block_pos.y, block_pos.z);
        }

        if !hit_block {
            None
        } else {
            Some(RaycastHit {
                distance: last_dist_to_next,
                position: block_pos,
                last_position: last_pos,
            })
        }
    }

    pub fn get_spawn_position(&self, rng: &mut Rng) -> Option<cgmath::Vector3<f32>> {
        let i_chunk_len = CHUNK_LEN as i32;
        let i_chunk_size = CHUNK_SIZE as i32;
        let i_chunk_height = CHUNK_HEIGHT as i32;

        let start_i = rng.range(CHUNK_LEN as u32) as i32;

        for offset_i in 0..i_chunk_len {
            let i = (start_i + offset_i) % i_chunk_len;
            let x = i % i_chunk_size;
            let y = (i / i_chunk_size) % i_chunk_height;
            let z = i / (i_chunk_size * i_chunk_height);

            if self.get_block(x, y, z) {
                continue;
            }

            return Some(cgmath::vec3(
                (x as f32 + 0.5) * BLOCK_SIZE_F,
                (y as f32 + 0.5) * BLOCK_SIZE_F,
                (z as f32 + 0.5) * BLOCK_SIZE_F,
            ));
        }

        None
    }

    pub fn model(&self) -> &Option<Model> {
        &self.model
    }

    pub fn add_entity_to_block(&mut self, entity: usize, x: i32, z: i32) {
        let i_chunk_size = CHUNK_SIZE as i32;
        if x < 0 || x >= i_chunk_size || z < 0 || z >= i_chunk_size {
            return;
        }

        let ux = x as usize;
        let uz = z as usize;

        self.entities_on_blocks[ux + uz * CHUNK_SIZE].insert(entity);
    }

    pub fn remove_entity_from_block(&mut self, entity: usize, x: i32, z: i32) {
        let i_chunk_size = CHUNK_SIZE as i32;
        if x < 0 || x >= i_chunk_size || z < 0 || z >= i_chunk_size {
            return;
        }

        let ux = x as usize;
        let uz = z as usize;

        self.entities_on_blocks[ux + uz * CHUNK_SIZE].remove(&entity);
    }

    pub fn entities_at_block(
        &self,
        x: i32,
        z: i32,
    ) -> Option<std::collections::hash_set::Iter<'_, usize>> {
        let i_chunk_size = CHUNK_SIZE as i32;
        if x < 0 || x >= i_chunk_size || z < 0 || z >= i_chunk_size {
            return None;
        }

        let ux = x as usize;
        let uz = z as usize;

        Some(self.entities_on_blocks[ux + uz * CHUNK_SIZE].iter())
    }

    fn calculate_ao_level(neighbors: VertexNeighbors) -> i32 {
        if neighbors.side1 && neighbors.side2 {
            return 0;
        }

        let mut occupied = 0;

        if neighbors.side1 {
            occupied += 1;
        }

        if neighbors.side2 {
            occupied += 1;
        }

        if neighbors.corner {
            occupied += 1;
        }

        3 - occupied
    }

    fn check_vertex_neighbors(
        &self,
        world_position: [i32; 3],
        vertex_position: [i32; 3],
        direction: Direction,
    ) -> VertexNeighbors {
        let direction_corner = [
            vertex_position[0] * 2 - 1,
            vertex_position[1] * 2 - 1,
            vertex_position[2] * 2 - 1,
        ];

        let outward_component = dir_outward_component(direction);
        let mut direction_side1 = direction_corner;
        direction_side1[((outward_component + 2) % 3) as usize] = 0;
        let mut direction_side2 = direction_corner;
        direction_side2[((outward_component + 1) % 3) as usize] = 0;

        let side1_position = [
            world_position[0] + direction_side1[0],
            world_position[1] + direction_side1[1],
            world_position[2] + direction_side1[2],
        ];
        let side2_position = [
            world_position[0] + direction_side2[0],
            world_position[1] + direction_side2[1],
            world_position[2] + direction_side2[2],
        ];
        let corner_position = [
            world_position[0] + direction_corner[0],
            world_position[1] + direction_corner[1],
            world_position[2] + direction_corner[2],
        ];

        VertexNeighbors {
            side1: self.get_block(side1_position[0], side1_position[1], side1_position[2]),
            side2: self.get_block(side2_position[0], side2_position[1], side2_position[2]),
            corner: self.get_block(corner_position[0], corner_position[1], corner_position[2]),
        }
    }

    fn orient_last_face(&mut self) {
        let face_start = self.vertices.len() - 4;
        let v0 = self.vertices[face_start];
        let v1 = self.vertices[face_start + 1];
        let v2 = self.vertices[face_start + 2];
        let v3 = self.vertices[face_start + 3];

        if self.ao_buffer[0] + self.ao_buffer[2] > self.ao_buffer[1] + self.ao_buffer[3] {
            return;
        }

        self.vertices[face_start] = v3;
        self.vertices[face_start + 1] = v0;
        self.vertices[face_start + 2] = v1;
        self.vertices[face_start + 3] = v2;
    }
}
