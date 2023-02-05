use crate::cube_mesh::CUBE_VERTICES;
use crate::direction::{index_to_dir, dir_to_offset};
use crate::instance::Instance;
use crate::model::Model;
use crate::rng::Rng;
use crate::{cube_mesh::CUBE_INDICES, vertex::Vertex};
use cgmath::Zero;

const CHUNK_SIZE: usize = 32;
const CHUNK_HEIGHT: usize = 8;
const CHUNK_LEN: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;
const BLOCK_SIZE: f32 = 3.0;

pub struct Chunk {
    pub model: Option<Model>,
    blocks: [bool; CHUNK_LEN],
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: [false; CHUNK_LEN],
            model: None,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn generate_blocks(&mut self, rng: &mut Rng) {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for x in 0..CHUNK_SIZE {
                    if rng.range(100) < 50 {
                        continue;
                    }

                    self.set_block(true, x as i32, y as i32, z as i32);
                }
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
                            vert.position[0] = (vert.position[0] + ix as f32) * BLOCK_SIZE;
                            vert.position[1] = (vert.position[1] + iy as f32) * BLOCK_SIZE;
                            vert.position[2] = (vert.position[2] + iz as f32) * BLOCK_SIZE;
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
                }],
            );
        }
    }

    pub fn set_block(&mut self, solid: bool, x: i32, y: i32, z: i32) {
        let i_chunk_size = CHUNK_SIZE as i32;
        let i_chunk_height = CHUNK_HEIGHT as i32;
        if x < 0 || x >= i_chunk_size || y < 0 || y >= i_chunk_height || z < 0 || z >= i_chunk_size {
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
        if x < 0 || x >= i_chunk_size || y < 0 || y >= i_chunk_height || z < 0 || z >= i_chunk_size {
            return false;
        }

        let ux = x as usize;
        let uy = y as usize;
        let uz = z as usize;

        self.blocks[ux + uy * CHUNK_SIZE + uz * CHUNK_SIZE * CHUNK_HEIGHT]
    }
}
