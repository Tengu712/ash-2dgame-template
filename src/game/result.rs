use super::*;
use crate::effect::{TextHorizontalAlignment, TextVerticalAlignment};
use glam::{Vec3, Vec4};

pub struct States {
    score: usize,
}

pub fn init(score: usize) -> States {
    States { score }
}

pub fn update(states: States, istates: &InputStates, effects: &mut Vec<Effect>) -> GameState {
    if istates.get(Key::Return) == 1 {
        return title::update(title::init(), istates, effects);
    }

    effects.push(Effect::DrawText {
        scale: 32,
        text: format!("You broke {} bricks!", states.score),
        halign: TextHorizontalAlignment::Center,
        valign: TextVerticalAlignment::Center,
        position: Vec3::new(VIRTUAL_WIDTH_HALF, VIRTUAL_HEIGHT_HALF, 0.0),
        line_height: 32.0,
        color: Vec4::new(1.0, 1.0, 1.0, 1.0),
    });

    GameState::Result(states)
}
