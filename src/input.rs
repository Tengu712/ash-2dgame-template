use crate::window::Window;
use std::collections::HashMap;

/// イテレーション可能なenumを定義するマクロ
macro_rules! define_iterable_enum {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),*
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];
        }
    };
}

define_iterable_enum!(Key { Return, Menu });

#[derive(Debug, Default)]
pub struct InputStates(pub HashMap<Key, usize>);

impl InputStates {
    pub fn update(&mut self, window: &Window) {
        for key in Key::ALL.iter() {
            if window.get_input_state(key.to_code()) {
                self.0.entry(*key).and_modify(|v| *v += 1).or_insert(1);
            } else {
                self.0.entry(*key).and_modify(|v| *v = 0).or_insert(0);
            }
        }
    }

    pub fn get(&self, key: Key) -> usize {
        self.0.get(&key).copied().unwrap_or(0)
    }
}

#[cfg(target_os = "windows")]
impl Key {
    fn to_code(self) -> u32 {
        match self {
            Self::Return => 0x0D,
            Self::Menu => 0x12,
        }
    }
}
