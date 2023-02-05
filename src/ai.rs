use crate::{input::Input, chunk::Chunk, actor::Actor};

pub trait Ai {
    fn update(&mut self, actor: &mut Actor, input: &mut Input, chunk: &Chunk, delta_time: f32);
}