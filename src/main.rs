use std::{
    cmp::Ordering,
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
use game::{Scene, World};
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
    let mut world = World::new(Scene::Title);
    let mut frame_start = Instant::now();

    while window.process_events() {
        istates = istates.update(&window);
        if istates.get(Key::Menu) > 0 && istates.get(Key::Return) == 1 {
            gengine = gengine.ensure_idle();
            window.toggle_fullscreen();
            gengine = gengine.recreate_swapchain(&window);
        }

        world.run(&istates);

        let (instances, camera) = collect_render_infos(&mut world);
        gengine = gengine.draw_frame(&window, instances, camera);

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

pub fn collect_render_infos(world: &mut World) -> (Vec<Instance>, Option<Camera>) {
    let mut instances = world
        .components
        .instances
        .0
        .values()
        .map(|n| n.data)
        .collect::<Vec<_>>();
    instances.sort_by(|a, b| {
        a.transform
            .w_axis
            .z
            .partial_cmp(&b.transform.w_axis.z)
            .unwrap_or(Ordering::Equal)
    });

    let camera = if world.camera_updated {
        world.camera_updated = false;
        Some(Camera {
            view: Mat4::IDENTITY,
            proj: Mat4::orthographic_rh(
                world.camera.left,
                world.camera.right,
                world.camera.bottom,
                world.camera.top,
                world.camera.near,
                world.camera.far,
            ),
        })
    } else {
        None
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
