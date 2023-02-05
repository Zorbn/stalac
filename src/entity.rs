use crate::{actor::Actor, ai::Ai, input::Input, chunk::Chunk};

pub struct Entity {
    pub actor: Actor,
    pub ai: Option<Box<dyn Ai>>
}

impl Entity {
    pub fn new(position: cgmath::Vector3<f32>, size: cgmath::Vector3<f32>, speed: f32, ai: Option<Box<dyn Ai>>) -> Self {
        Self {
            actor: Actor::new(
                position,
                size,
                speed,
            ),
            ai,
        }
    }

    pub fn update(&mut self, input: &mut Input, chunk: &Chunk, delta_time: f32) {
        let Entity { actor, ai } = self;

        actor.update(input, chunk, delta_time);

        if let Some(ai) = ai {
            ai.update(actor, input, chunk, delta_time);
        }
    }
}