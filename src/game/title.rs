use super::*;
use crate::{effect::TextAlignment, res::*};
use glam::{Vec2, Vec3, Vec4};

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
        return play::update(play::init(), istates, effects);
    }

    effects.push(Effect::Draw {
        position: Vec3::new(VIRTUAL_WIDTH_HALF, VIRTUAL_HEIGHT_HALF - 50.0, 0.0),
        scaling: Vec2::new(512.0, 64.0),
        color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        image: IMAGE,
        uv: UV_TITLE,
    });
    effects.push(Effect::DrawText {
        scale: 32,
        text: "PRESS ENTER KEY TO START".to_string(),
        align: TextAlignment::Center,
        position: Vec3::new(VIRTUAL_WIDTH_HALF, VIRTUAL_HEIGHT_HALF + 50.0, 0.0),
        line_height: 32.0,
        color: Vec4::new(1.0, 1.0, 1.0, 1.0),
    });

    GameState::Title(States {
        count: states.count + 1,
    })
}
