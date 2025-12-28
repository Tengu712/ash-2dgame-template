use super::{context::Context, window::Window};
use ash::{khr::surface, vk};
use std::{marker::PhantomData, sync::Arc};

/// スワップチェーン
///
/// Windowと同じスレッドでのみ動作する。
pub struct Swapchain<'a> {
    pub surface: vk::SurfaceKHR,
    ctx: Arc<Context>,
    _window: PhantomData<&'a ()>,
}

impl<'a> Swapchain<'a> {
    pub fn new(ctx: Arc<Context>, window: &'a Window) -> Self {
        let surface = window.create_surface(&ctx.entry, &ctx.instance);
        Self {
            surface,
            ctx,
            _window: PhantomData,
        }
    }
}

impl Drop for Swapchain<'_> {
    fn drop(&mut self) {
        unsafe {
            let instance = surface::Instance::new(&self.ctx.entry, &self.ctx.instance);
            instance.destroy_surface(self.surface, None);
        }
    }
}
