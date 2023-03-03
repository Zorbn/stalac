use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashSet,
};

use super::{
    actor::Actor,
    ecs::{Ecs, System},
    item::Item,
};

pub struct Inventory {
    items: Vec<char>,
}

impl Inventory {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    // TODO: Make item components return the parameters for this function.
    pub fn add_item(&mut self) {
        let c = if self.items.len() % 2 == 0 { 'a' } else { 'b' };
        self.items.push(c);
    }

    pub fn remove_item(&mut self, index: usize) {
        if index < self.items().len() {
            self.items.remove(index);
        }
    }

    pub fn items(&self) -> &Vec<char> {
        &self.items
    }
}

pub struct InventorySystem {
    nearby_entities: HashSet<usize>,
}

impl InventorySystem {
    pub fn new() -> Self {
        Self {
            nearby_entities: HashSet::new(),
        }
    }
}

impl System for InventorySystem {
    fn update(
        &mut self,
        ecs: &mut super::ecs::Ecs,
        chunk: &mut crate::chunk::Chunk,
        _input: &mut crate::input::Input,
        _gui: &mut crate::gfx::gui::Gui,
        _delta_time: f32,
    ) {
        let Ecs {
            manager,
            entity_cache,
            queue,
        } = ecs;

        if !manager.get_entities_with_both::<Inventory, Actor>(entity_cache) {
            return;
        }

        let actors = manager.borrow_components::<Actor>().unwrap();
        let mut inventories = manager.borrow_components::<Inventory>().unwrap();
        let items = match manager.borrow_components::<Item>() {
            Some(i) => i,
            None => return,
        };

        for entity in entity_cache {
            let actor = actors.borrow().get(*entity).unwrap();
            let inventory = inventories.borrow_mut().get_mut(*entity).unwrap();

            actor.get_nearby_entities(chunk, &mut self.nearby_entities);

            for nearby_entity in &self.nearby_entities {
                if *nearby_entity == *entity {
                    continue;
                }

                if !items.has(*nearby_entity) {
                    continue;
                }

                let nearby_actor = match actors.borrow().get(*nearby_entity) {
                    Some(a) => a,
                    None => continue,
                };

                if !nearby_actor.intersects(actor.position(), actor.size()) {
                    continue;
                }

                inventory.add_item();
                queue.remove_entity(*nearby_entity);
            }
        }
    }
}
