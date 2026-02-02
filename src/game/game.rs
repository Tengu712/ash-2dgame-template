use super::*;
use crate::input::Key;

const BAR_Y: f32 = 0.75;
const BAR_SIZE: (f32, f32) = (0.25, 0.05);

struct Bar {
    x: f32,
}

impl Bar {
    fn update(self, istates: &InputStates) -> Self {
        let r = (istates.get(Key::Right) > 0) as i32;
        let l = (istates.get(Key::Left) > 0) as i32;
        let dx = (r - l) as f32 * 0.025;
        Self { x: self.x + dx }
    }
}

pub struct States {
    bar: Bar,
}

impl States {
    pub fn new() -> Self {
        Self {
            bar: Bar { x: 0.0 },
        }
    }

    pub fn update(self, istates: &InputStates) -> (GameState, RenderingInfo) {
        let bar = self.bar.update(istates);
        let state = Self { bar };

        let instances = vec![Instance {
            position: Vec3::new(state.bar.x, BAR_Y, 0.0),
            scaling: Vec3::new(BAR_SIZE.0, BAR_SIZE.1, 1.0),
            ..Default::default()
        }];
        let rinfo = RenderingInfo {
            instances,
            ..Default::default()
        };

        (GameState::Game(Box::new(state)), rinfo)
    }
}
