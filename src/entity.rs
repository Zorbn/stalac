use crate::{actor::Actor, chunk::Chunk, input::Input, entities::Entities, chase_ai::ChaseAi, player_ai::PlayerAi};

// TODO: Convert to structure of arrays.
pub struct Entity {
    pub actor: Actor,
    pub chase_ai: Option<ChaseAi>,
    pub player_ai: Option<PlayerAi>,
}

impl Entity {
    pub fn new(
        position: cgmath::Vector3<f32>,
        size: cgmath::Vector3<f32>,
        speed: f32,
        chase_ai: Option<ChaseAi>,
        player_ai: Option<PlayerAi>,
    ) -> Self {
        Self {
            actor: Actor::new(position, size, speed),
            chase_ai,
            player_ai,
        }
    }

    // Todo: Change player position to entity list & player entity_id, once that system is implemented.
    pub fn update(
        self_id: u32,
        input: &mut Input,
        entities: &mut Entities,
        player_id: u32,
        chunk: &Chunk,
        delta_time: f32,
    ) {
        Actor::update(self_id, entities, input, chunk, delta_time);
        ChaseAi::update(self_id, input, entities, player_id, chunk, delta_time);
        PlayerAi::update(self_id, input, entities, player_id, chunk, delta_time);
    }
}
