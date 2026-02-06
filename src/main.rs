use std::{
    hint, thread,
    time::{Duration, Instant},
};

mod config;
mod game;
mod graphics;
mod logs;
mod res;
mod window;

use config::*;
use game::GameState;
use glam::*;
use graphics::{GraphicsEngine, descriptor::transform::*};
use res::*;
use window::{
    Window,
    input::{InputStates, Key},
};

enum Effect {
    Draw {
        position: Vec3,
        scaling: Vec2,
        color: Vec4,
    },
    LoadImage(Resource),
    UpdateCamera {
        position: Vec3,
        scaling: Vec3,
    },
}

fn main() {
    #[cfg(target_os = "macos")]
    set_env_for_molten_vk();

    let window = Window::new("ash-2dgame-template", SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut gengine = GraphicsEngine::new(&window);
    let mut istates = InputStates::default();
    let mut gstate = GameState::default();
    let mut frame_start = Instant::now();

    // NOTE: パフォーマンスのための可変変数
    let mut effects_buf = Vec::new();
    let mut instances = Vec::new();

    while window.process_events() {
        // 入力状態更新
        istates = istates.update(&window);

        // フルスクリーン/ウィンドウ切替え
        if istates.get(Key::Menu) > 0 && istates.get(Key::Return) == 1 {
            gengine = gengine.ensure_idle();
            window.toggle_fullscreen();
            gengine = gengine.recreate_swapchain(&window);
        }

        // ゲーム状態更新
        gstate = gstate.update(&istates, &mut effects_buf);

        // 副作用処理
        let mut camera = None;
        instances.clear();
        for effect in effects_buf.drain(..) {
            match effect {
                Effect::Draw {
                    position,
                    scaling,
                    color,
                } => instances.push(Instance {
                    transform: Mat4::from_translation(position)
                        * Mat4::from_scale(scaling.extend(1.0)),
                    color,
                }),
                Effect::LoadImage(res) => gengine = gengine.load_image(&res),
                Effect::UpdateCamera { position, scaling } => {
                    camera = Some(Camera {
                        view: Mat4::from_translation(-position),
                        proj: Mat4::from_scale(scaling.recip()),
                    })
                }
            }
        }

        // 描画
        gengine = gengine.draw_frame(&window, &instances, &camera);

        // 60FPS制限
        frame_start = sync_60fps(frame_start);
    }

    gengine.destroy();
    window.destroy();
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
