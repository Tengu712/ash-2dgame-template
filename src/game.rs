use crate::input::InputStates;
use std::{collections::HashMap, mem};

mod component;
mod system;

use component::ComponentStorages;
use system::{SetupSystem, System};

type Entity = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scene {
    Title,
    Play,
    Result,
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

pub struct World {
    pub next_entity: Entity,
    pub components: ComponentStorages,
    pub systems: Vec<System>,

    pub setup_systems: HashMap<Scene, SetupSystem>,
    pub next_scene: Option<Scene>,

    pub camera: Camera,
    pub camera_updated: bool,
}

impl World {
    pub fn new(initial_scene: Scene) -> Self {
        Self {
            next_entity: 0,
            components: ComponentStorages::default(),
            systems: Vec::new(),
            setup_systems: system::create_setup_systems(),
            next_scene: Some(initial_scene),
            camera: Camera {
                left: 0.0,
                right: 640.0,
                bottom: 0.0,
                top: 480.0,
                near: 0.0,
                far: 100.0,
            },
            camera_updated: true,
        }
    }

    pub fn run(&mut self, input_states: &InputStates) {
        if let Some(next_scene) = self.next_scene.take()
            && let Some(setup_system) = self.setup_systems.get(&next_scene)
        {
            setup_system(self);
        }
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

    fn clear(&mut self) {
        self.next_entity = 0;
        self.components.clear();
        self.systems.clear();
    }
}
