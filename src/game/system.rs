use super::World;
use crate::input::InputStates;

pub type System = fn(&mut World, &InputStates);

pub fn create_systems() -> Vec<System> {
    // NOTE: Systemは順番に実行されるので、追加順序に注意。
    vec![]
}
