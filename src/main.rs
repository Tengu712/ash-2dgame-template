use ash::Entry;
use std::sync::Arc;

mod graphics;
mod settings;

use graphics::{context::Context, submit::Submitter, swapchain::Swapchain, window::Window};
use settings::*;

fn main() {
    let window = Window::new(WINDOW_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT);

    let entry = Entry::linked();
    let ctx = Context::new(Arc::new(entry), APPLICATION_NAME, APPLICATION_VERSION);
    let ctx = Arc::new(ctx);
    let _ = Swapchain::new(Arc::clone(&ctx), &window);
    let _ = Submitter::new(Arc::clone(&ctx));

    while window.process_events() {
        // DEBUG:
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
