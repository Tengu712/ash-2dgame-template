use super::*;
use std::f32::consts;

pub fn clamp_and_reflect(world: &mut World, _: &InputStates) {
    for (_, p, v) in world.components.ball_position_velocity() {
        if p.x <= 0.0 {
            p.x = 0.0;
            v.t = consts::PI - v.t;
        } else if p.x >= 640.0 {
            p.x = 640.0;
            v.t = consts::PI - v.t;
        }

        if p.y <= 0.0 {
            p.y = 0.0;
            v.t = -v.t;
        } else if p.y >= 480.0 {
            p.y = 480.0;
            v.t = -v.t;
        }
    }
}
