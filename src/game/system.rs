use super::{Scene, World};
use crate::{
    game::component::*,
    input::{InputStates, Key},
};
use std::collections::HashMap;

pub type System = fn(&mut World, &InputStates);
pub type SetupSystem = fn(&mut World);

pub fn create_systems() -> Vec<System> {
    // NOTE: Systemは順番に実行されるので、追加順序に注意。
    vec![update_player, update_position_with_velocity]
}

pub fn create_setup_systems() -> HashMap<Scene, SetupSystem> {
    [(Scene::Title, setup_title_scene as SetupSystem)]
        .into_iter()
        .collect()
}

fn setup_title_scene(world: &mut World) {
    let entity = world.spawn();
    world
        .components
        .positions
        .0
        .insert(entity, Position { x: 0.0, y: 0.0 });
    world
        .components
        .velocities
        .0
        .insert(entity, Velocity { r: 0.0, t: 0.0 });
}

fn update_player(world: &mut World, inputs: &InputStates) {
    let Some(v) = world.components.velocities.0.get_mut(&0) else {
        return;
    };
    if inputs.get(Key::Return) > 0 {
        v.r = 1.0;
        v.t = 0.0;
    } else {
        v.r = 0.0;
    }
}

fn update_position_with_velocity(world: &mut World, _: &InputStates) {
    for (k, v) in world.components.velocities.0.iter() {
        if let Some(p) = world.components.positions.0.get_mut(k) {
            p.x += v.r * v.t.cos();
            p.y += v.r * v.t.sin();
        }
    }
}
