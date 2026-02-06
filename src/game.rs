use crate::{Effect, window::input::*};

pub mod game;
pub mod title;

const VIRTUAL_WIDTH: f32 = 640.0;
const VIRTUAL_HEIGHT: f32 = 480.0;
const VIRTUAL_WIDTH_HALF: f32 = VIRTUAL_WIDTH / 2.0;
const VIRTUAL_HEIGHT_HALF: f32 = VIRTUAL_HEIGHT / 2.0;

pub enum GameState {
    Title(title::States),
    Game(game::States),
}

impl GameState {
    pub fn update(self, istates: &InputStates, effects: &mut Vec<Effect>) -> Self {
        match self {
            Self::Title(states) => title::update(states, istates, effects),
            Self::Game(states) => game::update(states, istates, effects),
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::Title(title::init())
    }
}
