use super::*;

pub fn update(world: &mut World, inputs: &InputStates) {
    for (_, v) in world.components.player_velocity() {
        let l = inputs.get(Key::Left) > 0;
        let r = inputs.get(Key::Right) > 0;
        if l && !r {
            v.r = 5.0;
            v.t = 180.0_f32.to_radians();
        } else if !l && r {
            v.r = 5.0;
            v.t = 0.0;
        } else {
            v.r = 0.0;
        }
    }
}

pub fn clamp_position(world: &mut World, _: &InputStates) {
    for (_, p) in world.components.player_position() {
        p.x = p.x.clamp(40.0, 640.0 - 40.0);
    }
}
