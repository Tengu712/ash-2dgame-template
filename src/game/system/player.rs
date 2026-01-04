use super::*;

pub fn update(world: &mut World, inputs: &InputStates) {
    for k in world.components.players.0.keys() {
        let Some(v) = world.components.velocities.0.get_mut(k) else {
            continue;
        };
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
