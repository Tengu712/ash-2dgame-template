use ash::{prelude::VkResult, vk};
use std::{ffi::CStr, rc::Rc};

mod graphics;
mod input;
mod logs;
mod window;

use graphics::{
    context::Context, framebuffer::Framebuffer, renderpass::RenderPass, submit::Submitter,
    swapchain::Swapchain,
};
use input::{InputStates, Key};
use logs::*;
use window::Window;

fn main() {
    const WINDOW_TITLE: &str = "ash-2dgame-template";
    const SCREEN_WIDTH: u32 = 1280;
    const SCREEN_HEIGHT: u32 = 720;

    const APPLICATION_NAME: &CStr = c"ash-2dgame-template";
    const APPLICATION_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);

    // ロガー初期化
    logs::setup_logger();

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
    let mut render_pass = RenderPass::new(
        Rc::clone(&ctx),
        swapchain.image_views.len(),
        swapchain.format.format,
    )
    .expect_log("failed to create a render pass");
    let mut framebuffers = Framebuffer::from_swapchain(&ctx, render_pass.render_pass, &swapchain)
        .expect_log("failed to create framebuffers");

    // メインループ
    while window.process_events() {
        // 入力状態更新
        input_states.update(&window);

        // 描画
        let result = toggle_fullscreen_if_needed(&window, &input_states, &ctx);
        let result = result.and_then(|_| {
            render_frame(&mut submitter, &swapchain, &mut render_pass, &framebuffers)
        });
        match result {
            Ok(()) => (),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
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
    }

    let _ = ctx.wait_idle();
}

fn toggle_fullscreen_if_needed(
    window: &Window,
    input_states: &InputStates,
    ctx: &Context,
) -> VkResult<()> {
    if input_states.get(Key::Menu) > 0 && input_states.get(Key::Return) == 1 {
        ctx.wait_idle()?;
        window.toggle_fullscreen();
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR)
    } else {
        Ok(())
    }
}

fn render_frame<'a>(
    submitter: &mut Submitter,
    swapchain: &'a Swapchain,
    render_pass: &mut RenderPass,
    framebuffers: &[Framebuffer<'a>],
) -> VkResult<()> {
    // 準備
    let recording_render_pass = render_pass.prepare();
    let index = swapchain
        .acquire_next_image_index(recording_render_pass.swapchain_image_started_semaphore())?;
    let framebuffer = &framebuffers[index as usize];
    let area = vk::Rect2D {
        offset: vk::Offset2D::default(),
        extent: swapchain.resolution,
    };

    // 記録&提出
    let command_buffer = submitter.prepare()?;
    let semaphores = recording_render_pass.record_render_commands(
        command_buffer.command_buffer(),
        framebuffer,
        area,
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
}
