use super::*;

pub fn update_position_with_velocity(world: &mut World, _: &InputStates) {
    for (p, v) in world.components.position_velocity() {
        p.x += v.r * v.t.cos();
        p.y += v.r * v.t.sin();
    }
}
