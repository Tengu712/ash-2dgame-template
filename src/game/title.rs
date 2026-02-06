use super::*;
use crate::res::IMAGE;
use glam::Vec3;

pub struct States {
    count: usize,
}

pub fn init() -> States {
    States { count: 0 }
}

pub fn update(states: States, istates: &InputStates, effects: &mut Vec<Effect>) -> GameState {
    if states.count == 0 {
        effects.push(Effect::LoadImage(IMAGE));
        effects.push(Effect::UpdateCamera {
            position: Vec3::new(VIRTUAL_WIDTH_HALF, VIRTUAL_HEIGHT_HALF, 0.0),
            scaling: Vec3::new(VIRTUAL_WIDTH_HALF, VIRTUAL_HEIGHT_HALF, 1.0),
        });
    }

    if istates.get(Key::Return) == 1 {
        return game::update(game::init(), istates, effects);
    }

    GameState::Title(States {
        count: states.count + 1,
    })
}
