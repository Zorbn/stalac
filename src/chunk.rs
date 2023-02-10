use std::collections::HashSet;

use crate::direction::{dir_to_offset, index_to_dir};
use crate::gfx::cube_mesh::{CUBE_INDICES, CUBE_VERTICES};
use crate::gfx::instance::Instance;
use crate::gfx::model::Model;
use crate::gfx::vertex::Vertex;
use crate::rng::Rng;
use cgmath::Zero;

pub const BLOCK_SIZE: i32 = 3;
pub const BLOCK_SIZE_F: f32 = BLOCK_SIZE as f32;
const CHUNK_SIZE: usize = 32;
const CHUNK_HEIGHT: usize = 8;
const CHUNK_LEN: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;
const INV_BLOCK_SIZE: f32 = 1.0 / BLOCK_SIZE as f32;

pub struct Chunk {
    model: Option<Model>,
    blocks: [bool; CHUNK_LEN],
    entities_on_blocks: Vec<HashSet<usize>>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
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

    pub fn generate_mesh(&mut self, device: &wgpu::Device) {
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
                        if self.get_block(ix + dir_offset.0, iy + dir_offset.1, iz + dir_offset.2) {
                            continue;
                        }

                        let vert_count = self.vertices.len() as u32;

                        for vert_i in 0..4 {
                            let mut vert = CUBE_VERTICES[dir_i][vert_i];
                            vert.position[0] = (vert.position[0] + ix as f32) * BLOCK_SIZE_F;
                            vert.position[1] = (vert.position[1] + iy as f32) * BLOCK_SIZE_F;
                            vert.position[2] = (vert.position[2] + iz as f32) * BLOCK_SIZE_F;
                            self.vertices.push(vert);
                        }

                        for ind_i in 0..6 {
                            self.indices.push(CUBE_INDICES[dir_i][ind_i] + vert_count);
                        }
                    }
                }
            }
        }

        self.model = Some(Model::new(device, &self.vertices, &self.indices));

        if let Some(model) = &mut self.model {
            model.update_instances(
                &device,
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

        let steps = size.cast::<i32>().expect("Failed to calculate step count!")
            + cgmath::Vector3::new(1, 1, 1);
        let mut interp = cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0);

        for x in 0..=steps.x {
            interp.x = start.x + x as f32 / steps.x as f32 * size.x;
            for y in 0..=steps.y {
                interp.y = start.y + y as f32 / steps.y as f32 * size.y;
                for z in 0..=steps.z {
                    interp.z = start.z + z as f32 / steps.z as f32 * size.z;

                    let interp_block = interp
                        .cast::<i32>()
                        .expect("Failed to calculate interp block!");

                    if self.get_block(interp_block.x, interp_block.y, interp_block.z) {
                        return Some(interp_block);
                    }
                }
            }
        }

        None
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

            return Some(cgmath::Vector3::new(
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
}
