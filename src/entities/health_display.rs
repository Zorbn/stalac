use std::borrow::Borrow;

use crate::{gfx::gui::Gui, input::Input, chunk::Chunk};

use super::{ecs::{System, Ecs}, health::Health};

pub struct HealthDisplay {}

pub struct HealthDisplaySystem {}

impl System for HealthDisplaySystem {
    fn update(
        &mut self,
        ecs: &mut Ecs,
        _chunk: &mut Chunk,
        _input: &mut Input,
        gui: &mut Gui,
        _delta_time: f32,
    ) {
        let Ecs { manager, entity_cache, .. } = ecs;

        if !manager.get_entities_with_both::<Health, HealthDisplay>(entity_cache) {
            return;
        }

        let healths = manager.borrow_components::<Health>().unwrap();

        for entity in entity_cache {
            let health = healths.borrow().get(*entity).unwrap();
            gui.write(&format!("Health: {}", health.amount()));
        }
    }
}
