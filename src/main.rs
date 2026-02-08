use std::{
    hint, thread,
    time::{Duration, Instant},
};

mod config;
mod effect;
mod game;
mod graphics;
mod logs;
mod res;
mod window;

use config::*;
use effect::{Effect, EffectProcessor};
use game::GameState;
use graphics::GraphicsEngine;
use window::{
    Window,
    input::{InputStates, Key},
};

struct System {
    window: Window,
    gengine: GraphicsEngine,
}

impl System {
    pub fn new() -> Self {
        let window = Window::new("ash-2dgame-template", SCREEN_WIDTH, SCREEN_HEIGHT);
        let gengine = GraphicsEngine::new(&window);
        Self { window, gengine }
    }

    pub fn destroy(self) {
        self.gengine.destroy();
        self.window.destroy();
    }
}

fn main() {
    #[cfg(target_os = "macos")]
    set_env_for_molten_vk();

    let mut system = System::new();
    let mut istates = InputStates::default();
    let mut gstate = GameState::default();
    let mut processor = EffectProcessor::new(&system);
    let mut effects = Vec::new();
    let mut frame_start = Instant::now();

    while system.window.process_events() {
        // 入力状態更新
        istates = istates.update(&system.window);

        // フルスクリーン/ウィンドウ切替え
        if istates.get(Key::Menu) > 0 && istates.get(Key::Return) == 1 {
            effects.push(Effect::ToggleFullscreen);
        }

        // ゲーム状態更新
        gstate = gstate.update(&istates, &mut effects);

        // 副作用処理
        (processor, system) = processor.process(effects.drain(..), system);

        // 60FPS制限
        frame_start = sync_60fps(frame_start);
    }

    system.destroy();
}

#[cfg(target_os = "macos")]
fn set_env_for_molten_vk() {
    use crate::logs::*;
    use std::env;

    let exe_dir = env::current_exe().expect_log("failed to get the execution file path");
    let exe_dir = exe_dir
        .parent()
        .expect_log("failed to get the execution directory path");
    unsafe {
        env::set_var("VK_ICD_FILENAMES", exe_dir);
        env::set_var("MVK_CONFIG_LOG_LEVEL", "0");
    }
}

pub fn sync_60fps(start: Instant) -> Instant {
    const TARGET_FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60); // 60FPS
    const SPIN_THRESHOLD: Duration = Duration::from_millis(2);

    let elapsed = start.elapsed();
    if elapsed < TARGET_FRAME_TIME {
        let remaining = TARGET_FRAME_TIME - elapsed;
        if remaining > SPIN_THRESHOLD {
            thread::sleep(remaining - SPIN_THRESHOLD);
        }
        while start.elapsed() < TARGET_FRAME_TIME {
            hint::spin_loop();
        }
    }

    Instant::now()
}
