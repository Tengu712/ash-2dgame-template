use ash::Entry;
use std::ffi::CStr;

mod context;

use context::Context;

pub fn create_entry() -> Entry {
    unsafe {
        Entry::load().expect(
            "failed to load Vulkan loader.\nPlease verify that Vulkan is available on your system.",
        )
    }
}

pub struct GraphicsEngine {
    ctx: Context,
}

impl GraphicsEngine {
    pub fn new(entry: &Entry, app_name: &CStr, app_version: u32) -> Self {
        let ctx = Context::new(entry, app_name, app_version);
        Self { ctx }
    }
}
