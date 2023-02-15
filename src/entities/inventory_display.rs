use std::borrow::Borrow;

use super::{ecs::{System, Ecs}, inventory::Inventory};

pub struct InventoryDisplay {

}

pub struct InventoryDisplaySystem {
    string: String,
}

impl InventoryDisplaySystem {
    pub fn new() -> Self {
        Self {
            string: String::new(),
        }
    }
}

impl System for InventoryDisplaySystem {
    fn update(
        &mut self,
        ecs: &mut super::ecs::Ecs,
        _chunk: &mut crate::chunk::Chunk,
        _input: &mut crate::input::Input,
        gui: &mut crate::gfx::gui::Gui,
        _delta_time: f32,
    ) {
        let Ecs {
            manager,
            entity_cache,
            ..
        } = ecs;

        if !manager.get_entities_with_both::<InventoryDisplay, Inventory>(entity_cache) {
            return;
        }

        let inventories = manager.borrow_components::<Inventory>().unwrap();

        for entity in entity_cache {
            let inventory = inventories.borrow().get(*entity).unwrap();

            self.string.clear();
            for item in inventory.items() {
                self.string.push(*item);
            }

            gui.write(&self.string);
        }
    }
}