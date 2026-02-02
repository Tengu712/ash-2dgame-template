use super::*;
use crate::input::Key;

pub struct States;

impl States {
    pub fn new() -> Self {
        Self
    }

    pub fn update(self, istates: &InputStates) -> (GameState, RenderingInfo) {
        if istates.get(Key::Return) == 1 {
            return game::States::new().update(istates);
        }

        (GameState::Title(Box::new(self)), RenderingInfo::default())
    }
}
