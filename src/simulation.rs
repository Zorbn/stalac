use crate::chunk::Chunk;
use crate::entities::actor::{Actor, ActorSystem};
use crate::entities::chase_ai::{ChaseAi, ChaseAiSystem};
use crate::entities::display::Display;
use crate::entities::ecs::{CommandQueue, Ecs, EntityManager, SystemManager};
use crate::entities::entity_instances_system::EntityInstancesSystem;
use crate::entities::fighter::{Fighter, FighterSystem};
use crate::entities::health::{Health, HealthSystem};
use crate::entities::health_display::{HealthDisplay, HealthDisplaySystem};
use crate::entities::inventory::{Inventory, InventorySystem};
use crate::entities::inventory_display::{InventoryDisplay, InventoryDisplaySystem};
use crate::entities::item::Item;
use crate::entities::player::{Player, PlayerMovementSystem};
use crate::gfx::gui::Gui;
use crate::gfx::instance::Instance;
use crate::input::Input;
use crate::rng::Rng;
use cgmath::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

const HUMANOID_SIZE: cgmath::Vector3<f32> = cgmath::vec3(1.0, 1.0, 1.0);
const ITEM_SIZE: cgmath::Vector3<f32> = cgmath::vec3(1.0, 1.0, 1.0);

pub struct Simulation {
    pub chunk: Chunk,
    ecs: Ecs,
    systems: SystemManager,
    player: usize,
    gui: Gui,
}

impl Simulation {
    pub fn new() -> Self {
        let mut rng = Rng::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Inaccurate system time!")
                .as_millis() as u32,
        );
        let mut chunk = Chunk::new();
        chunk.generate_blocks(&mut rng);

        let mut ecs = Ecs {
            manager: EntityManager::new(),
            queue: CommandQueue::new(),
            entity_cache: Vec::new(),
        };

        let mut player_actor = Actor::new(cgmath::Vector3::zero(), HUMANOID_SIZE, 6.0);

        if let Some(player_spawn) = chunk.get_spawn_position(&mut rng) {
            player_actor.teleport(player_spawn);
        }

        let mut enemy_actor = Actor::new(cgmath::Vector3::zero(), HUMANOID_SIZE, 6.0);

        if let Some(enemy_spawn) = chunk.get_spawn_position(&mut rng) {
            enemy_actor.teleport(enemy_spawn);
        }

        let player = ecs.manager.add_entity();
        ecs.manager.add_component_to_entity(player, player_actor);
        ecs.manager.add_component_to_entity(player, Player {});
        ecs.manager
            .add_component_to_entity(player, Fighter::new(25, 0.25));
        ecs.manager
            .add_component_to_entity(player, Health::new(100));
        ecs.manager
            .add_component_to_entity(player, HealthDisplay {});
        ecs.manager
            .add_component_to_entity(player, Inventory::new());
        ecs.manager
            .add_component_to_entity(player, InventoryDisplay {});
        let enemy = ecs.manager.add_entity();
        ecs.manager.add_component_to_entity(enemy, enemy_actor);
        ecs.manager.add_component_to_entity(enemy, ChaseAi::new());
        ecs.manager.add_component_to_entity(enemy, Display::new(1));
        ecs.manager.add_component_to_entity(enemy, Health::new(50));
        ecs.manager
            .add_component_to_entity(enemy, Fighter::new(10, 0.5));

        for _ in 0..10 {
            if let Some(item_spawn) = chunk.get_spawn_position(&mut rng) {
                let test_item = ecs.manager.add_entity();
                ecs.manager
                    .add_component_to_entity(test_item, Actor::new(item_spawn, ITEM_SIZE, 0.0));
                ecs.manager
                    .add_component_to_entity(test_item, Display::new(0));
                ecs.manager.add_component_to_entity(test_item, Item {});
            }
        }

        let mut systems = SystemManager::new();
        systems.add_system(ActorSystem {});
        systems.add_system(ChaseAiSystem {});
        systems.add_system(PlayerMovementSystem {});
        systems.add_system(EntityInstancesSystem::new());
        systems.add_system(FighterSystem::new());
        systems.add_system(HealthDisplaySystem {});
        systems.add_system(HealthSystem {});
        systems.add_system(InventorySystem::new());
        systems.add_system(InventoryDisplaySystem::new());

        let gui = Gui::new();

        Self {
            chunk,
            ecs,
            systems,
            player,
            gui,
        }
    }

    pub fn update(&mut self, input: &mut Input, delta_time: f32) {
        self.gui.clear();
        self.ecs.flush_queue(&mut self.chunk);

        self.systems.update(
            &mut self.ecs,
            &mut self.chunk,
            input,
            &mut self.gui,
            delta_time,
        );
    }

    pub fn entity_instances(&self) -> &Vec<Instance> {
        self.systems
            .get::<EntityInstancesSystem>()
            .unwrap()
            .instances()
    }

    pub fn gui_instances(&self) -> &Vec<Instance> {
        self.gui.instances()
    }

    pub fn ecs(&self) -> &Ecs {
        &self.ecs
    }

    pub fn focused_entity(&self) -> usize {
        self.player
    }
}
