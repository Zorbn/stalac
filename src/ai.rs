use crate::{actor::Actor, chunk::Chunk, input::Input};

pub trait Ai {
    fn update(&mut self, actor: &mut Actor, input: &mut Input, player_position: cgmath::Vector3<f32>, chunk: &Chunk, delta_time: f32);
}
