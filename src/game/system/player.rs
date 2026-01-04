use super::*;

pub fn update(world: &mut World, inputs: &InputStates) {
    for k in world.components.players.0.keys() {
        let Some(v) = world.components.velocities.0.get_mut(k) else {
            continue;
        };
        // TODO:
        if inputs.get(Key::Return) > 0 {
            v.r = 1.0;
            v.t = 0.0;
        } else {
            v.r = 0.0;
        }
    }
}
