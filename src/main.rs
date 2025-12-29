use ash::vk;
use std::{ffi::CStr, rc::Rc};

mod graphics;
mod window;

use graphics::{context::Context, submit::Submitter, swapchain::Swapchain};
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
    let _ = Swapchain::new(Rc::clone(&ctx), Rc::clone(&window));
    let _ = Submitter::new(Rc::clone(&ctx));

    while window.process_events() {
        // DEBUG:
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
