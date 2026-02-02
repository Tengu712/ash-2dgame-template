use crate::input::InputStates;
use glam::{Vec2, Vec3, Vec4};

pub mod game;
pub mod title;

const VIRTUAL_WIDTH: f32 = 640.0;
const VIRTUAL_HEIGHT: f32 = 480.0;
const VIRTUAL_WIDTH_HALF: f32 = VIRTUAL_WIDTH / 2.0;
const VIRTUAL_HEIGHT_HALF: f32 = VIRTUAL_HEIGHT / 2.0;

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    pub position: Vec3,
    pub scaling: Vec2,
    pub color: Vec4,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            scaling: Vec2::new(1.0, 1.0),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub scaling: Vec3,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(VIRTUAL_WIDTH_HALF, VIRTUAL_HEIGHT_HALF, 0.0),
            scaling: Vec3::new(VIRTUAL_WIDTH_HALF, VIRTUAL_HEIGHT_HALF, 1.0),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct RenderingInfo {
    pub instances: Vec<Instance>,
    pub camera: Camera,
}

pub enum GameState {
    Title(Box<title::States>),
    Game(Box<game::States>),
}

impl GameState {
    pub fn update(self, istates: &InputStates) -> (Self, RenderingInfo) {
        match self {
            Self::Title(state) => state.update(istates),
            Self::Game(state) => state.update(istates),
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::Title(Box::new(title::States::new()))
    }
}
