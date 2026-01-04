use crate::{
    graphics::descriptor::transform::{Camera, Instance},
    input::InputStates,
};
use glam::{Mat4, Vec3, Vec4};
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

    pub fn collect_render_infos(&self) -> (Vec<Instance>, Camera) {
        // TODO:
        let instances = self
            .components
            .positions
            .0
            .values()
            .map(|pos| Instance {
                transform: Mat4::from_scale(Vec3::new(640.0, 480.0, 1.0)),
                color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            })
            .collect();
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
