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

define_iterable_enum!(Key {
    Left,
    Right,
    Return,
    Menu
});

#[cfg(target_os = "windows")]
impl Key {
    fn to_code(self) -> u32 {
        match self {
            Self::Left => 0x25,
            Self::Right => 0x27,
            Self::Return => 0x0D,
            Self::Menu => 0x12,
        }
    }
}

#[cfg(target_os = "macos")]
impl Key {
    fn to_code(self) -> u16 {
        match self {
            Self::Left => 123,
            Self::Right => 124,
            Self::Return => 36,
            Self::Menu => 999,
        }
    }
}

#[cfg(target_os = "linux")]
impl Key {
    fn to_code(self) -> u32 {
        match self {
            Self::Left => 113,
            Self::Right => 114,
            Self::Return => 36,
            Self::Menu => 64,
        }
    }
}

#[derive(Debug, Default)]
pub struct InputStates(pub HashMap<Key, usize>);

impl InputStates {
    pub fn update(self, window: &Window) -> Self {
        let mut states = self.0;
        for key in Key::ALL.iter() {
            if window.get_input_state(key.to_code()) {
                states.entry(*key).and_modify(|v| *v += 1).or_insert(1);
            } else {
                states.entry(*key).and_modify(|v| *v = 0).or_insert(0);
            }
        }
        Self(states)
    }

    pub fn get(&self, key: Key) -> usize {
        self.0.get(&key).copied().unwrap_or(0)
    }
}
