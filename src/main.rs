use ash::{prelude::VkResult, vk};
use std::{
    ffi::CStr,
    hint,
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

mod game;
mod graphics;
mod input;
mod logs;
mod window;

use game::{Scene, World};
use graphics::{
    context::Context,
    descriptor::Descriptors,
    framebuffer::Framebuffer,
    renderpass::{RenderAreas, RenderPass},
    submit::Submitter,
    swapchain::Swapchain,
};
use input::{InputStates, Key};
use logs::*;
use window::Window;

fn main() {
    const WINDOW_TITLE: &str = "ash-2dgame-template";
    const SCREEN_WIDTH: u32 = 640;
    const SCREEN_HEIGHT: u32 = 480;
    const ASPECT_RATIO: f32 = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;

    const APPLICATION_NAME: &CStr = c"ash-2dgame-template";
    const APPLICATION_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);

    const MAX_INSTANCE_COUNT: usize = 32;

    const TARGET_FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60); // 60FPS
    const SPIN_THRESHOLD: Duration = Duration::from_millis(2);

    // ウィンドウ作成
    let window = Window::new(WINDOW_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT);
    let window = Rc::new(window);

    // 入力状態管理オブジェクト作成
    let mut input_states = InputStates::default();

    // 描画用オブジェクト作成
    let ctx = Context::new(APPLICATION_NAME, APPLICATION_VERSION);
    let ctx = Rc::new(ctx);
    let mut submitter = Submitter::new(Rc::clone(&ctx));
    let mut swapchain = Swapchain::new(Rc::clone(&ctx), Rc::clone(&window));
    let descriptors = Descriptors::new(Rc::clone(&ctx), MAX_INSTANCE_COUNT);
    let mut render_pass = RenderPass::new(
        Rc::clone(&ctx),
        swapchain.image_views.len(),
        swapchain.format.format,
        &descriptors,
    )
    .expect_log("failed to create a render pass");
    let mut framebuffers = Framebuffer::from_swapchain(&ctx, render_pass.render_pass, &swapchain)
        .expect_log("failed to create framebuffers");

    // ゲームオブジェクト作成
    let mut world = World::new(Scene::Title);

    // メインループ
    let mut frame_start = Instant::now();
    while window.process_events() {
        // 入力状態更新
        input_states.update(&window);

        // ゲーム更新
        world.run(&input_states);

        // 描画
        //
        // NOTE: エラーをキャプチャするためにクロージャで実行。
        let result = || -> VkResult<()> {
            // フルスクリーン/ウィンドウ切替え確認
            if input_states.get(Key::Menu) > 0 && input_states.get(Key::Return) == 1 {
                ctx.wait_idle()?;
                window.toggle_fullscreen();
                return Err(vk::Result::ERROR_OUT_OF_DATE_KHR);
            }

            // 準備
            let semaphores = render_pass.semaphores();
            let index = swapchain.acquire_next_image_index(semaphores.started_semaphore)?;
            let framebuffer = &framebuffers[index as usize];
            let render_area = swapchain.get_full_rect();
            let areas = RenderAreas {
                render_area,
                viewport: swapchain.calc_aspect_corrected_viewport(ASPECT_RATIO),
                scissor: render_area,
            };

            // 前回提出したコマンドの実行完了を待機
            submitter.wait()?;

            // ディスクリプタ更新
            let (instances, camera) = world.collect_render_infos();
            if !instances.is_empty() {
                descriptors
                    .trans
                    .insts_buffer
                    .copy_to_memory(&instances[..instances.len().min(MAX_INSTANCE_COUNT)], 0)?;
            }
            if let Some(camera) = camera {
                descriptors.trans.camera_buffer.copy_to_memory(&camera)?;
            }

            // 記録&提出
            let command_buffer = submitter.prepare()?;
            descriptors
                .record_bind_command(command_buffer.command_buffer(), render_pass.pipeline.layout);
            render_pass.record_render_commands(
                command_buffer.command_buffer(),
                framebuffer,
                areas,
                instances.len(),
            )?;
            command_buffer.submit(
                &[(
                    semaphores.started_semaphore,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                )],
                &[semaphores.finished_semaphore],
            )?;

            // プレゼンテーション
            swapchain.queue_presentation_command(index, semaphores.finished_semaphore)?;

            Ok(())
        }();

        // エラーキャプチャ
        match result {
            Ok(()) => (),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                // スワップチェーン再作成
                drop(framebuffers);
                swapchain = swapchain
                    .recreate()
                    .expect_log("failed to recreate a swapchain");
                framebuffers =
                    Framebuffer::from_swapchain(&ctx, render_pass.render_pass, &swapchain)
                        .expect_log("failed to recreate framebuffers");
            }
            Err(e) => panic_log(&format!("unrecoverable error occurred: {e}")),
        }

        // フレームレート制限
        let frame_time = frame_start.elapsed();
        if frame_time < TARGET_FRAME_TIME {
            let remaining = TARGET_FRAME_TIME - frame_time;
            if remaining > SPIN_THRESHOLD {
                thread::sleep(remaining - SPIN_THRESHOLD);
            }
            while frame_start.elapsed() < TARGET_FRAME_TIME {
                hint::spin_loop();
            }
        }
        frame_start = Instant::now();
    }

    let _ = ctx.wait_idle();
}
