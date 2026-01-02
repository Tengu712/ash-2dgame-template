use ash::vk;
use std::{ffi::CStr, rc::Rc};

mod graphics;
mod window;

use graphics::{
    context::Context, framebuffer::Framebuffer, renderpass::RenderPass, submit::Submitter,
    swapchain::Swapchain,
};
use window::Window;

fn main() {
    const WINDOW_TITLE: &str = "ash-2dgame-template";
    const SCREEN_WIDTH: u32 = 1280;
    const SCREEN_HEIGHT: u32 = 720;

    const APPLICATION_NAME: &CStr = c"ash-2dgame-template";
    const APPLICATION_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);

    let window = Window::new(WINDOW_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT);
    let window = Rc::new(window);

    let ctx = Context::new(APPLICATION_NAME, APPLICATION_VERSION);
    let ctx = Rc::new(ctx);
    let swapchain = Swapchain::new(Rc::clone(&ctx), Rc::clone(&window));
    let mut render_pass = RenderPass::new(
        Rc::clone(&ctx),
        swapchain.image_views.len(),
        swapchain.format.format,
    )
    .expect("failed to create a render pass");
    let framebuffers = Framebuffer::from_swapchain(&ctx, render_pass.render_pass, &swapchain)
        .expect("failed to create framebuffers");
    let mut submitter = Submitter::new(Rc::clone(&ctx));

    while window.process_events() {
        if window.get_input_state(0x0D) {
            window.toggle_fullscreen();
        }

        // TODO: エラー復帰

        let recording_render_pass = render_pass.prepare();
        let index = swapchain
            .acquire_next_image_index(recording_render_pass.swapchain_image_started_semaphore())
            .expect("failed to acquire next swapchain image index");
        let framebuffer = &framebuffers[index as usize];
        let area = vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: swapchain.resolution,
        };

        let command_buffer = submitter
            .prepare()
            .expect("failed to prepare recording rendering commands");
        let semaphores = recording_render_pass
            .record_render_commands(command_buffer.command_buffer(), framebuffer, area)
            .expect("failed to recording rendering commands");

        let wait_infos = [(
            semaphores.started_semaphore,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        )];
        let signal_semaphores = [semaphores.finished_semaphore];
        command_buffer
            .submit(&wait_infos, &signal_semaphores)
            .expect("failed to submit rendering commands");

        swapchain
            .queue_presentation_command(index, semaphores.finished_semaphore)
            .expect("failed to queue a presentation command");
    }

    let _ = ctx.wait_idle();
}
