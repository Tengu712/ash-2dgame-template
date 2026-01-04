use crate::{
    graphics::descriptor::transform::{Camera, Instance},
    input::InputStates,
};
use glam::{Mat4, Vec3, Vec4};
use std::{collections::HashMap, mem};

mod component;
mod system;

use component::ComponentStorages;
use system::{SetupSystem, System};

type Entity = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scene {
    Title,
}

pub struct World {
    next_entity: Entity,
    components: ComponentStorages,
    systems: Vec<System>,
    setup_systems: HashMap<Scene, SetupSystem>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            components: ComponentStorages::default(),
            systems: system::create_systems(),
            setup_systems: system::create_setup_systems(),
        }
    }

    pub fn load_scene(&mut self, scene: Scene) {
        if let Some(setup_system) = self.setup_systems.get(&scene) {
            setup_system(self);
        }
    }

    pub fn run(&mut self, input_states: &InputStates) {
        let systems = mem::take(&mut self.systems);
        for system in &systems {
            system(self, input_states);
        }
        self.systems = systems;
    }

    pub fn collect_render_infos(&self, max_instance_count: usize) -> (Vec<Instance>, Camera) {
        // TODO:
        let mut instances = Vec::with_capacity(max_instance_count);
        for (_, pos) in self.components.positions.0.iter() {
            instances.push(Instance {
                transform: Mat4::from_translation(Vec3::new(pos.x, pos.y, 0.0))
                    * Mat4::from_scale(Vec3::new(640.0, 480.0, 1.0)),
                color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            });
        }
        let camera = Camera {
            view: Mat4::IDENTITY,
            proj: Mat4::orthographic_rh(0.0, 640.0, 0.0, 480.0, 0.0, 100.0),
        };
        (instances, camera)
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
