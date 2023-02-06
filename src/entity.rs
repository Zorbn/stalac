use crate::{actor::Actor, ai::Ai, chunk::Chunk, input::Input};

pub struct Entity {
    pub actor: Actor,
    pub ai: Option<Box<dyn Ai>>,
}

impl Entity {
    pub fn new(
        position: cgmath::Vector3<f32>,
        size: cgmath::Vector3<f32>,
        speed: f32,
        ai: Option<Box<dyn Ai>>,
    ) -> Self {
        Self {
            actor: Actor::new(position, size, speed),
            ai,
        }
    }

    // Todo: Change player position to player entity_id, once that system is implemented.
    pub fn update(&mut self, input: &mut Input, player_position: cgmath::Vector3<f32>, chunk: &Chunk, delta_time: f32) {
        let Entity { actor, ai } = self;

        actor.update(input, chunk, delta_time);

        if let Some(ai) = ai {
            ai.update(actor, input, player_position, chunk, delta_time);
        }
    }
}
