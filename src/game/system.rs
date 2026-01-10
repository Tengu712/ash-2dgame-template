use super::{Scene, World, component::*};
use crate::input::{InputStates, Key};
use std::collections::HashMap;

pub mod ball;
pub mod instance;
pub mod player;
pub mod scene_play;
pub mod scene_result;
pub mod scene_title;
pub mod transform;

pub type System = fn(&mut World, &InputStates);
pub type SetupSystem = fn(&mut World);

pub fn create_setup_systems() -> HashMap<Scene, SetupSystem> {
    [
        (Scene::Title, scene_title::setup as SetupSystem),
        (Scene::Play, scene_play::setup as SetupSystem),
        (Scene::Result, scene_result::setup as SetupSystem),
    ]
    .into_iter()
    .collect()
}
