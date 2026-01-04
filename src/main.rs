use ash::{prelude::VkResult, vk};
use std::{ffi::CStr, rc::Rc};

mod game;
mod graphics;
mod input;
mod logs;
mod window;

use game::{Scene, World};
use graphics::{
    context::Context, descriptor::Descriptors, framebuffer::Framebuffer, renderpass::RenderPass,
    submit::Submitter, swapchain::Swapchain,
};
use input::{InputStates, Key};
use logs::*;
use window::Window;

fn main() {
    const WINDOW_TITLE: &str = "ash-2dgame-template";
    const SCREEN_WIDTH: u32 = 640;
    const SCREEN_HEIGHT: u32 = 480;

    const APPLICATION_NAME: &CStr = c"ash-2dgame-template";
    const APPLICATION_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);

    const MAX_INSTANCE_COUNT: usize = 32;

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
    let mut world = World::new();
    world.load_scene(Scene::Title);

    // メインループ
    while window.process_events() {
        // 入力状態更新
        input_states.update(&window);

        // ゲーム更新
        world.run(&input_states);

        // 描画
        let result = toggle_fullscreen_if_needed(&window, &input_states, &ctx);
        let result =
            result.and_then(|_| update_descriptors(&world, &descriptors, MAX_INSTANCE_COUNT));
        let result = result.and_then(|count| {
            render_frame(
                &mut submitter,
                &swapchain,
                &mut render_pass,
                &framebuffers,
                &descriptors,
                count,
            )
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

fn update_descriptors(
    world: &World,
    descriptors: &Descriptors,
    max_instance_count: usize,
) -> VkResult<usize> {
    let (instances, camera) = world.collect_render_infos(max_instance_count);
    if !instances.is_empty() {
        descriptors
            .trans
            .insts_buffer
            .copy_to_memory(&instances, 0)?;
    }
    descriptors.trans.camera_buffer.copy_to_memory(&camera)?;
    Ok(instances.len())
}

fn render_frame<'a>(
    submitter: &mut Submitter,
    swapchain: &'a Swapchain,
    render_pass: &mut RenderPass,
    framebuffers: &[Framebuffer<'a>],
    descriptors: &Descriptors,
    count: usize,
) -> VkResult<()> {
    // 準備
    let semaphores = render_pass.semaphores();
    let index = swapchain.acquire_next_image_index(semaphores.started_semaphore)?;
    let framebuffer = &framebuffers[index as usize];
    let area = vk::Rect2D {
        offset: vk::Offset2D::default(),
        extent: swapchain.resolution,
    };

    // 記録&提出
    let command_buffer = submitter.prepare()?;
    descriptors.record_bind_command(command_buffer.command_buffer(), render_pass.pipeline.layout);
    render_pass.record_render_commands(
        command_buffer.command_buffer(),
        framebuffer,
        area,
        count,
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
