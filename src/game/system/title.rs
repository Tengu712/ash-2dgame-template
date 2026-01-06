use super::*;

pub fn setup(world: &mut World) {
    world.clear();

    // components
    // - player
    let entity = world.spawn();
    world.components.players.0.insert(entity, Player {});
    world
        .components
        .instances
        .0
        .insert(entity, InstanceData::default());
    world
        .components
        .positions
        .0
        .insert(entity, Position { x: 320.0, y: 400.0 });
    world
        .components
        .velocities
        .0
        .insert(entity, Velocity { r: 0.0, t: 0.0 });
    world
        .components
        .scales
        .0
        .insert(entity, Scale { x: 10.0, y: 10.0 });

    // systems
    world.systems.push(player::update);
    world.systems.push(transform::update_position_with_velocity);
    world.systems.push(instance::update_instance);
}
