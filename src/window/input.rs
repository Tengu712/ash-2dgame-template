use super::Window;
use std::collections::HashMap;

macro_rules! define_key {
    ($(($variant:ident, $win:literal, $mac:literal, $lin:literal),)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Key {
            $($variant),*
        }

        impl Key {
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];

            fn to_code(self) -> u32 {
                #[cfg(target_os = "windows")]
                match self { $(Self::$variant => $win,)* }
                #[cfg(target_os = "macos")]
                match self { $(Self::$variant => $mac,)* }
                #[cfg(target_os = "linux")]
                match self { $(Self::$variant => $lin,)* }
            }
        }
    };
}

define_key!(
    (Left, 0x25, 123, 113),
    (Right, 0x27, 124, 114),
    (Return, 0x0D, 36, 36),
    (Menu, 0x12, 999, 64),
);

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
