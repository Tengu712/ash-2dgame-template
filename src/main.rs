use ash::vk;
use fern::Dispatch;
use std::{ffi::CStr, fmt::Display, rc::Rc, time::SystemTime};

mod graphics;
mod window;

use graphics::{
    context::Context, framebuffer::Framebuffer, renderpass::RenderPass, submit::Submitter,
    swapchain::Swapchain,
};
use window::Window;

trait LoggableResult<T, E> {
    /// `Err`であればロギングとエラーダイアログ表示を行う`unwrap()`
    fn expect_log(self, message: &str) -> T;
}

impl<T, E: Display> LoggableResult<T, E> for Result<T, E> {
    fn expect_log(self, message: &str) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                let message = format!("{message}: {e}");
                log::error!("{message}");
                Window::show_error_dialog(&message);
                panic!("{message}");
            }
        }
    }
}

fn main() {
    const WINDOW_TITLE: &str = "ash-2dgame-template";
    const SCREEN_WIDTH: u32 = 1280;
    const SCREEN_HEIGHT: u32 = 720;

    const APPLICATION_NAME: &CStr = c"ash-2dgame-template";
    const APPLICATION_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);

    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                humantime::format_rfc3339(SystemTime::now()),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file("log.txt").expect_log("failed to chain log.txt"))
        .apply()
        .expect_log("failed to initialize logger");

    let window = Window::new(WINDOW_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT);
    let window = Rc::new(window);

    let ctx = Context::new(APPLICATION_NAME, APPLICATION_VERSION);
    let ctx = Rc::new(ctx);
    let mut swapchain = Swapchain::new(Rc::clone(&ctx), Rc::clone(&window));
    let mut render_pass = RenderPass::new(
        Rc::clone(&ctx),
        swapchain.image_views.len(),
        swapchain.format.format,
    )
    .expect("failed to create a render pass");
    let mut framebuffers = Framebuffer::from_swapchain(&ctx, render_pass.render_pass, &swapchain)
        .expect("failed to create framebuffers");
    let mut submitter = Submitter::new(Rc::clone(&ctx));

    let mut return_key_down_count = 0;
    while window.process_events() {
        if window.get_input_state(0x0D) {
            if return_key_down_count == 0 {
                ctx.wait_idle().expect("failed to wait for idle");
                window.toggle_fullscreen();
                drop(framebuffers);
                swapchain = swapchain
                    .recreate()
                    .expect("failed to recreate a swapchain");
                framebuffers =
                    Framebuffer::from_swapchain(&ctx, render_pass.render_pass, &swapchain)
                        .expect("failed to recreate framebuffers");
            }
            return_key_down_count += 1;
        } else {
            return_key_down_count = 0;
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
