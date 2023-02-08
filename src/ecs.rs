use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

pub struct Ecs {
    entities_count: usize,
    component_stores: Vec<Box<dyn ComponentManager>>,
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            entities_count: 0,
            component_stores: Vec::new(),
        }
    }

    pub fn add_entity(&mut self) -> usize {
        let entity_id = self.entities_count;
        self.entities_count += 1;
        entity_id
    }

    pub fn remove_entity<T: 'static>(&mut self, entity_id: usize) {
        for component_store in self.component_stores.iter_mut() {
            component_store.remove(entity_id);
        }
    }

    pub fn add_component_to_entity<T: 'static>(&mut self, entity_id: usize, component: T) {
        for component_store in self.component_stores.iter_mut() {
            if let Some(component_store) = component_store
                .as_any_mut()
                .downcast_mut::<RefCell<ComponentStore<T>>>()
            {
                component_store.get_mut().add(entity_id, component);
                return;
            }
        }

        // There wasn't an existing place to store this component, create one.
        let mut new_component_store = ComponentStore::<T>::new();
        new_component_store.add(entity_id, component);
        self.component_stores
            .push(Box::new(RefCell::new(new_component_store)));
    }

    pub fn remove_component_from_entity<T: 'static>(&mut self, entity_id: usize) {
        for component_store in self.component_stores.iter_mut() {
            if let Some(component_store) = component_store
                .as_any_mut()
                .downcast_mut::<RefCell<ComponentStore<T>>>()
            {
                component_store.get_mut().remove(entity_id);
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

    pub fn get_ids_with<T1: 'static, T2: 'static>(&self) -> Vec<usize> {
        let first_store = self.borrow_components::<T1>();
        let second_store = self.borrow_components::<T2>();

        let mut entity_id_cache = Vec::new(); // TODO: Pass this to function.

        entity_id_cache.clear();

        if first_store.is_none() || second_store.is_none() {
            return entity_id_cache;
        }

        let first_store = first_store.unwrap();
        let second_store = second_store.unwrap();

        for entity_id in &first_store.entity_ids {
            if second_store.has(*entity_id) {
                entity_id_cache.push(*entity_id);
            }
        }

        entity_id_cache
    }
}

pub struct ComponentStore<T> {
    components: Vec<T>,
    entity_ids: Vec<usize>,
    entity_id_map: HashMap<usize, usize>,
}

impl<T> ComponentStore<T> {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            entity_ids: Vec::new(),
            entity_id_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, entity_id: usize, component: T) {
        if self.has(entity_id) {
            return;
        }

        self.components.push(component);
        self.entity_ids.push(entity_id);
        self.entity_id_map
            .insert(entity_id, self.components.len() - 1);
    }

    pub fn remove(&mut self, entity_id: usize) {
        if !self.has(entity_id) {
            return;
        }

        let index = *self.entity_id_map.get(&entity_id).unwrap();
        // Remove this component, and move the last component in the store into its place.
        self.entity_id_map
            .insert(*self.entity_ids.last().unwrap(), index);
        self.components.swap_remove(index);
        self.entity_ids.swap_remove(index);
        self.entity_id_map.remove(&entity_id);
    }

    pub fn has(&self, entity_id: usize) -> bool {
        self.entity_id_map.contains_key(&entity_id)
    }

    pub fn get(&mut self, entity_id: usize) -> Option<&mut T> {
        let index = self.entity_id_map.get(&entity_id).unwrap();
        return self.components.get_mut(*index);
    }

    pub fn get_all(&mut self) -> &mut Vec<T> {
        &mut self.components
    }
}

pub trait ComponentManager {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn remove(&mut self, entity_id: usize);
}

impl<T: 'static> ComponentManager for RefCell<ComponentStore<T>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn remove(&mut self, entity_id: usize) {
        self.get_mut().remove(entity_id);
    }
}
