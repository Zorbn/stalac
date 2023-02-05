use crate::cube_mesh::CUBE_VERTICES;
use crate::direction::{Direction, index_to_dir, dir_to_offset};
use crate::{cube_mesh::CUBE_INDICES, vertex::Vertex};
use rand::prelude::*;

const CHUNK_SIZE: usize = 16;
const CHUNK_LEN: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub struct Chunk {
    blocks: [bool; CHUNK_LEN],
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: [false; CHUNK_LEN],
        }
    }

    pub fn generate_blocks(&mut self) {
        let mut rng = rand::thread_rng();

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if rng.gen::<f32>() < 0.5 {
                        continue;
                    }

                    self.set_block(true, x as i32, y as i32, z as i32);
                }
            }
        }
    }

    pub fn generate_mesh(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for z in 0..CHUNK_SIZE {
            let iz = z as i32;
            for y in 0..CHUNK_SIZE {
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

                        let vert_count = vertices.len() as u32;

                        for vert_i in 0..4 {
                            let mut vert = CUBE_VERTICES[dir_i][vert_i];
                            vert.position[0] += ix as f32;
                            vert.position[1] += iy as f32;
                            vert.position[2] += iz as f32;
                            vertices.push(vert);
                        }

                        for ind_i in 0..6 {
                            indices.push(CUBE_INDICES[dir_i][ind_i] + vert_count);
                        }
                    }
                }
            }
        }

        (vertices, indices)
    }

    pub fn set_block(&mut self, solid: bool, x: i32, y: i32, z: i32) {
        let i_chunk_size = CHUNK_SIZE as i32;
        if x < 0 || x >= i_chunk_size || y < 0 || y >= i_chunk_size || z < 0 || z >= i_chunk_size {
            return;
        }

        let ux = x as usize;
        let uy = y as usize;
        let uz = z as usize;

        self.blocks[ux + uy * CHUNK_SIZE + uz * CHUNK_SIZE * CHUNK_SIZE] = solid;
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> bool {
        let i_chunk_size = CHUNK_SIZE as i32;
        if x < 0 || x >= i_chunk_size || y < 0 || y >= i_chunk_size || z < 0 || z >= i_chunk_size {
            return false;
        }

        let ux = x as usize;
        let uy = y as usize;
        let uz = z as usize;

        self.blocks[ux + uy * CHUNK_SIZE + uz * CHUNK_SIZE * CHUNK_SIZE]
    }
}
