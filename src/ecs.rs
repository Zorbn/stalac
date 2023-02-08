use std::{
    cell::{RefCell, RefMut},
    collections::HashMap, ops::Deref,
};

use crate::entity_instances_system::EntityInstancesSystem;

pub struct EntityManager {
    entities_count: usize,
    component_managers: Vec<Box<dyn ComponentManager>>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities_count: 0,
            component_managers: Vec::new(),
        }
    }

    pub fn add_entity(&mut self) -> usize {
        let entity = self.entities_count;
        self.entities_count += 1;
        entity
    }

    pub fn remove_entity<T: 'static>(&mut self, entity: usize) {
        for component_store in self.component_managers.iter_mut() {
            component_store.remove(entity);
        }
    }

    pub fn add_component_to_entity<T: 'static>(&mut self, entity: usize, component: T) {
        for component_store in self.component_managers.iter_mut() {
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
        self.component_managers
            .push(Box::new(RefCell::new(new_component_store)));
    }

    pub fn remove_component_from_entity<T: 'static>(&mut self, entity: usize) {
        for component_store in self.component_managers.iter_mut() {
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
        for component_store in self.component_managers.iter() {
            if let Some(component_store) = component_store
                .as_any()
                .downcast_ref::<RefCell<ComponentStore<T>>>()
            {
                return Some(component_store.borrow_mut());
            }
        }

        None
    }

    pub fn get_entities_with<T1: 'static, T2: 'static>(&self, entities: &mut Vec<usize>) {
        let first_store = self.borrow_components::<T1>();
        let second_store = self.borrow_components::<T2>();

        entities.clear();

        if first_store.is_none() || second_store.is_none() {
            return;
        }

        let first_store = first_store.unwrap();
        let second_store = second_store.unwrap();

        for entity in &first_store.entities {
            if second_store.has(*entity) {
                entities.push(*entity);
            }
        }
    }
}

pub struct SystemManager {
    systems: Vec<Box<dyn SystemStoreTrait>>,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        ecs: &mut EntityManager,
        entity_cache: &mut Vec<usize>,
        chunk: &crate::chunk::Chunk,
        input: &mut crate::input::Input,
        player: usize,
        delta_time: f32,
    ) {
        for system in &mut self.systems {
            system.update(ecs, entity_cache, chunk, input, player, delta_time);
        }
    }

    pub fn add_system<T: 'static + System>(&mut self, system: T) {
        self.systems.push(Box::new(SystemStore::new(system)));
    }

    pub fn get<T: 'static>(&mut self) -> Option<&T> {
        for system in self.systems.iter() {
            if let Some(system) = system.as_any()
                .downcast_ref::<SystemStore<T>>()
            {
                return Some(&system.system); // TODO: Fix all the stupid naming for system stuff (system.system names of system traits and structs).
            }
        }

        None
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

    pub fn get(&mut self, entity: usize) -> Option<&mut T> {
        let index = self.entity_map.get(&entity).unwrap();
        return self.components.get_mut(*index);
    }

    pub fn get_all(&mut self) -> &mut Vec<T> {
        &mut self.components
    }
}

pub trait ComponentManager {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn remove(&mut self, entity: usize);
}

impl<T: 'static> ComponentManager for RefCell<ComponentStore<T>> {
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

pub trait System {
    fn update(
        &mut self,
        ecs: &mut EntityManager,
        entity_cache: &mut Vec<usize>,
        chunk: &crate::chunk::Chunk,
        input: &mut crate::input::Input,
        player: usize,
        delta_time: f32,
    );
}

pub trait SystemStoreTrait {
    fn as_any(&self) -> &dyn std::any::Any;

    fn update(
        &mut self,
        ecs: &mut EntityManager,
        entity_cache: &mut Vec<usize>,
        chunk: &crate::chunk::Chunk,
        input: &mut crate::input::Input,
        player: usize,
        delta_time: f32,
    );
}

pub struct SystemStore<T> {
    system: T,
}

impl<T: System> SystemStore<T> {
    pub fn new(system: T) -> Self {
        Self {
            system,
        }
    }
}

impl<T: 'static + System> SystemStoreTrait for SystemStore<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn update(
        &mut self,
        ecs: &mut EntityManager,
        entity_cache: &mut Vec<usize>,
        chunk: &crate::chunk::Chunk,
        input: &mut crate::input::Input,
        player: usize,
        delta_time: f32,
    ) {
        self.system.update(ecs, entity_cache, chunk, input, player, delta_time);
    }
}