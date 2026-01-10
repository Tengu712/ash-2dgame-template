use super::*;

pub fn setup(world: &mut World) {
    world.clear();

    // components

    // systems
    world.systems.push(to_next_scene);
}

fn to_next_scene(world: &mut World, inputs: &InputStates) {
    if inputs.get(Key::Return) == 1 {
        world.next_scene = Some(Scene::Play);
    }
}
