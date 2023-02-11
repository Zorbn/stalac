use std::borrow::Borrow;

use super::ecs::{Ecs, System};

pub struct Health {
    amount: i32,
}

impl Health {
    pub fn new(amount: i32) -> Self {
        Self { amount }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.amount -= amount;
    }

    pub fn amount(&self) -> i32 {
        self.amount
    }
}

pub struct HealthSystem {}

impl System for HealthSystem {
    fn update(
        &mut self,
        ecs: &mut super::ecs::Ecs,
        _chunk: &mut crate::chunk::Chunk,
        _input: &mut crate::input::Input,
        _gui: &mut crate::gfx::gui::Gui,
        _delta_time: f32,
    ) {
        let Ecs {
            manager,
            entity_cache,
            queue,
        } = ecs;

        if !manager.get_entities_with::<Health>(entity_cache) {
            return;
        }

        let healths = manager.borrow_components::<Health>().unwrap();

        for entity in entity_cache {
            let health = healths.borrow().get(*entity).unwrap();

            if health.amount() <= 0 {
                queue.remove_entity(*entity);
            }
        }
    }
}
