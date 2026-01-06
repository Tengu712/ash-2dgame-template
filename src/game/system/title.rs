use super::*;

pub fn setup(world: &mut World) {
    world.clear();

    // components
    // - player
    let entity = world.spawn();
    world.components.insert_player(entity, Player {});
    world
        .components
        .insert_instance(entity, InstanceData::default());
    world
        .components
        .insert_position(entity, Position { x: 320.0, y: 440.0 });
    world
        .components
        .insert_scale(entity, Scale { x: 80.0, y: 10.0 });
    world
        .components
        .insert_velocity(entity, Velocity { r: 0.0, t: 0.0 });

    // systems
    world.systems.push(player::update);
    world.systems.push(transform::update_position_with_velocity);
    world.systems.push(player::clamp_position);
    world.systems.push(instance::update_instance);
}
