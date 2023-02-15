use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use crate::{chunk::Chunk, gfx::gui::Gui, input::Input};

use super::actor::Actor;

pub struct Ecs {
    pub manager: EntityManager,
    pub queue: CommandQueue,
    pub entity_cache: Vec<usize>,
}

impl Ecs {
    pub fn flush_queue(&mut self, chunk: &mut Chunk) {
        let actors = match self.manager.borrow_components::<Actor>() {
            Some(a) => a,
            None => return,
        };

        for entity in self.queue.entities_to_remove() {
            // Remove actor from the map.
            actors
                .get(*entity)
                .unwrap()
                .update_occupied_blocks(*entity, chunk, None);
        }

        drop(actors);

        for entity in self.queue.entities_to_remove() {
            self.manager.remove_entity(*entity);
        }

        self.queue.clear();
    }
}

pub struct CommandQueue {
    entities_to_remove: Vec<usize>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            entities_to_remove: Vec::new(),
        }
    }

    pub fn remove_entity(&mut self, entity: usize) {
        self.entities_to_remove.push(entity);
    }

    pub fn clear(&mut self) {
        self.entities_to_remove.clear();
    }

    pub fn entities_to_remove(&self) -> &Vec<usize> {
        &self.entities_to_remove
    }
}

pub struct EntityManager {
    entities_count: usize,
    component_stores: Vec<Box<dyn AnyComponentStore>>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities_count: 0,
            component_stores: Vec::new(),
        }
    }

    pub fn add_entity(&mut self) -> usize {
        let entity = self.entities_count;
        self.entities_count += 1;
        entity
    }

    pub fn remove_entity(&mut self, entity: usize) {
        for component_store in self.component_stores.iter_mut() {
            component_store.remove(entity);
        }
    }

    pub fn add_component_to_entity<T: 'static>(&mut self, entity: usize, component: T) {
        for component_store in self.component_stores.iter_mut() {
            if let Some(component_store) = component_store
                .as_any_mut()
                .downcast_mut::<RefCell<ComponentStore<T>>>()
            {
                component_store.get_mut().add(entity, component);
                return;
            }
        }

        // There wasn't an existing place to store this component, create one.
        let mut new_component_store = ComponentStore::<T>::new();
        new_component_store.add(entity, component);
        self.component_stores
            .push(Box::new(RefCell::new(new_component_store)));
    }

    pub fn remove_component_from_entity<T: 'static>(&mut self, entity: usize) {
        for component_store in self.component_stores.iter_mut() {
            if let Some(component_store) = component_store
                .as_any_mut()
                .downcast_mut::<RefCell<ComponentStore<T>>>()
            {
                component_store.get_mut().remove(entity);
                return;
            }
        }
    }

    pub fn borrow_components<T: 'static>(&self) -> Option<RefMut<ComponentStore<T>>> {
        for component_store in self.component_stores.iter() {
            if let Some(component_store) = component_store
                .as_any()
                .downcast_ref::<RefCell<ComponentStore<T>>>()
            {
                return Some(component_store.borrow_mut());
            }
        }

        None
    }

    pub fn get_entities_with<T: 'static>(&self, entities: &mut Vec<usize>) -> bool {
        let store = self.borrow_components::<T>();

        entities.clear();

        if let Some(store) = store {
            entities.extend(store.get_entities());
        }

        !entities.is_empty()
    }

    pub fn get_entities_with_both<T1: 'static, T2: 'static>(
        &self,
        entities: &mut Vec<usize>,
    ) -> bool {
        let first_store = self.borrow_components::<T1>();
        let second_store = self.borrow_components::<T2>();

        entities.clear();

        if first_store.is_none() || second_store.is_none() {
            return false;
        }

        let first_store = first_store.unwrap();
        let second_store = second_store.unwrap();

        for entity in &first_store.entities {
            if second_store.has(*entity) {
                entities.push(*entity);
            }
        }

        !entities.is_empty()
    }
}

pub struct ComponentStore<T> {
    components: Vec<T>,
    entities: Vec<usize>,
    entity_map: HashMap<usize, usize>,
}

impl<T> ComponentStore<T> {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            entities: Vec::new(),
            entity_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, entity: usize, component: T) {
        if self.has(entity) {
            return;
        }

        self.components.push(component);
        self.entities.push(entity);
        self.entity_map.insert(entity, self.components.len() - 1);
    }

    pub fn remove(&mut self, entity: usize) {
        if !self.has(entity) {
            return;
        }

        let index = *self.entity_map.get(&entity).unwrap();
        // Remove this component, and move the last component in the store into its place.
        self.entity_map
            .insert(*self.entities.last().unwrap(), index);
        self.components.swap_remove(index);
        self.entities.swap_remove(index);
        self.entity_map.remove(&entity);
    }

    pub fn has(&self, entity: usize) -> bool {
        self.entity_map.contains_key(&entity)
    }

    pub fn get(&self, entity: usize) -> Option<&T> {
        let index = match self.entity_map.get(&entity) {
            Some(i) => i,
            None => return None,
        };
        self.components.get(*index)
    }

    pub fn get_all(&self) -> &Vec<T> {
        &self.components
    }

    pub fn get_mut(&mut self, entity: usize) -> Option<&mut T> {
        let index = match self.entity_map.get(&entity) {
            Some(i) => i,
            None => return None,
        };
        self.components.get_mut(*index)
    }

    pub fn get_all_mut(&mut self) -> &mut Vec<T> {
        &mut self.components
    }

    pub fn get_entities(&self) -> &Vec<usize> {
        &self.entities
    }
}

pub trait AnyComponentStore {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn remove(&mut self, entity: usize);
}

impl<T: 'static> AnyComponentStore for RefCell<ComponentStore<T>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn remove(&mut self, entity: usize) {
        self.get_mut().remove(entity);
    }
}

pub struct SystemManager {
    system_stores: Vec<Box<dyn AnySystemStore>>,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            system_stores: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        ecs: &mut Ecs,
        chunk: &mut Chunk,
        input: &mut Input,
        gui: &mut Gui,
        delta_time: f32,
    ) {
        for system_store in &mut self.system_stores {
            system_store.update(ecs, chunk, input, gui, delta_time);
        }
    }

    pub fn add_system<T: 'static + System>(&mut self, system: T) {
        self.system_stores.push(Box::new(SystemStore::new(system)));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        for system_store in self.system_stores.iter() {
            if let Some(system_store) = system_store.as_any().downcast_ref::<SystemStore<T>>() {
                return Some(&system_store.system);
            }
        }

        None
    }

    pub fn remove<T: 'static>(&mut self) {
        for (i, system_store) in self.system_stores.iter().enumerate() {
            if system_store
                .as_any()
                .downcast_ref::<SystemStore<T>>()
                .is_some()
            {
                self.system_stores.remove(i);
                return;
            }
        }
    }
}

pub trait System {
    fn update(
        &mut self,
        ecs: &mut Ecs,
        chunk: &mut Chunk,
        input: &mut Input,
        gui: &mut Gui,
        delta_time: f32,
    );
}

pub trait AnySystemStore {
    fn as_any(&self) -> &dyn std::any::Any;

    fn update(
        &mut self,
        ecs: &mut Ecs,
        chunk: &mut Chunk,
        input: &mut Input,
        gui: &mut Gui,
        delta_time: f32,
    );
}

pub struct SystemStore<T> {
    system: T,
}

impl<T: System> SystemStore<T> {
    pub fn new(system: T) -> Self {
        Self { system }
    }
}

impl<T: 'static + System> AnySystemStore for SystemStore<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn update(
        &mut self,
        ecs: &mut Ecs,
        chunk: &mut Chunk,
        input: &mut Input,
        gui: &mut Gui,
        delta_time: f32,
    ) {
        self.system.update(ecs, chunk, input, gui, delta_time);
    }
}
