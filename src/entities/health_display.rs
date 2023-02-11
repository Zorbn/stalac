use std::borrow::Borrow;

use super::{ecs::System, health::Health};

pub struct HealthDisplay {}

pub struct HealthDisplaySystem {}

impl System for HealthDisplaySystem {
    fn update(
        &mut self,
        ecs: &mut super::ecs::EntityManager,
        entity_cache: &mut Vec<usize>,
        _chunk: &mut crate::chunk::Chunk,
        _input: &mut crate::input::Input,
        gui: &mut crate::gfx::gui::Gui,
        _delta_time: f32,
    ) {
        ecs.get_entities_with_both::<Health, HealthDisplay>(entity_cache);

        if entity_cache.is_empty() {
            return;
        }

        let healths = ecs.borrow_components::<Health>().unwrap();

        for entity in entity_cache {
            let health = healths.borrow().get(*entity).unwrap();
            gui.write(&format!("Health: {}", health.amount()));
        }
    }
}
