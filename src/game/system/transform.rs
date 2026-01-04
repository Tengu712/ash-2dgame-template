use super::*;

pub fn update_position_with_velocity(world: &mut World, _: &InputStates) {
    for (k, v) in world.components.velocities.0.iter() {
        if let Some(p) = world.components.positions.0.get_mut(k) {
            p.x += v.r * v.t.cos();
            p.y += v.r * v.t.sin();
        }
    }
}
