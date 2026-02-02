use std::{
    hint, thread,
    time::{Duration, Instant},
};

mod config;
mod game;
mod graphics;
mod input;
mod logs;
mod window;

use config::*;
use game::{GameState, RenderingInfo};
use glam::Mat4;
use graphics::{GraphicsEngine, descriptor::transform::*};
use input::{InputStates, Key};
use window::Window;

fn main() {
    #[cfg(target_os = "macos")]
    set_env_for_molten_vk();

    let window = Window::new("ash-2dgame-template", SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut gengine = GraphicsEngine::new(&window);
    let mut istates = InputStates::default();
    let mut gstate = GameState::default();
    let mut camera = Camera {
        view: Mat4::ZERO,
        proj: Mat4::ZERO,
    };
    let mut frame_start = Instant::now();

    while window.process_events() {
        // 入力状態更新
        istates = istates.update(&window);

        // フルスクリーン/ウィンドウ切替え
        if istates.get(Key::Menu) > 0 && istates.get(Key::Return) == 1 {
            gengine = gengine.ensure_idle();
            window.toggle_fullscreen();
            gengine = gengine.recreate_swapchain(&window);
        }

        // ゲーム状態更新&描画情報取得
        let (ngstate, rinfo) = gstate.update(&istates);
        gstate = ngstate;

        // 描画
        let (instances, ncamera) = collect_render_infos(rinfo, camera);
        camera = ncamera.unwrap_or(camera);
        gengine = gengine.draw_frame(&window, instances, ncamera);

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

pub fn collect_render_infos(
    rinfo: RenderingInfo,
    camera_cache: Camera,
) -> (Vec<Instance>, Option<Camera>) {
    let instances = rinfo
        .instances
        .iter()
        .map(|instance| Instance {
            transform: Mat4::from_translation(instance.position)
                * Mat4::from_scale(instance.scaling),
            color: instance.color,
        })
        .collect();

    let camera = Camera {
        view: Mat4::from_translation(-rinfo.camera.position),
        proj: Mat4::from_scale(rinfo.camera.scaling.recip()),
    };
    let camera = if camera == camera_cache {
        None
    } else {
        Some(camera)
    };

    (instances, camera)
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
