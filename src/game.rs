use crate::input::InputStates;
use std::mem;

mod component;
mod system;

use component::ComponentStorages;
use system::System;

type Entity = usize;

pub struct World {
    next_entity: Entity,
    components: ComponentStorages,
    systems: Vec<System>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            components: ComponentStorages::default(),
            systems: system::create_systems(),
        }
    }

    pub fn run(&mut self, input_states: &InputStates) {
        let systems = mem::take(&mut self.systems);
        for system in &systems {
            system(self, input_states);
        }
        self.systems = systems;
    }

    fn spawn(&mut self) -> Entity {
        let entity = self.next_entity;
        self.next_entity += 1;
        entity
    }

    fn destroy(&mut self, entity: Entity) {
        self.components.destroy_entity(entity);
    }
}
