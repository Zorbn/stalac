use std::borrow::BorrowMut;

use crate::gfx::sprite_mesh::UI_SPRITE_WIDTH;

use super::{
    ecs::{Ecs, System},
    inventory::Inventory,
};

pub struct InventoryDisplay {}

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
        input: &mut crate::input::Input,
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

        let mut inventories = manager.borrow_components::<Inventory>().unwrap();

        for entity in entity_cache {
            let inventory = inventories.borrow_mut().get_mut(*entity).unwrap();

            self.string.clear();
            for item in inventory.items() {
                self.string.push(*item);
            }


            let height = gui.write(&self.string);
            let mouse_position = input.gui_mouse_position();
            if mouse_position.y >= height && mouse_position.y <= height + 1.0 {
                let hovered_index = (mouse_position.x / UI_SPRITE_WIDTH).floor() as usize;
                inventory.remove_item(hovered_index);
            }
            // println!("{:?}", input.gui_mouse_position());
        }
    }
}
