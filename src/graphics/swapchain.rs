use super::{context::Context, window::Window};
use ash::{
    khr::{surface, swapchain},
    prelude::VkResult,
    vk,
};
use std::sync::Arc;

mod info;

use info::SurfaceInfoForSwapchain;

/// スワップチェーン
///
/// Windowと同じスレッドでのみ動作する。
pub struct Swapchain<'a> {
    pub surface: vk::SurfaceKHR,
    pub resolution: vk::Extent2D,
    pub format: vk::SurfaceFormatKHR,
    pub swapchain: vk::SwapchainKHR,
    ctx: Arc<Context>,
    window: &'a Window,
}

impl<'a> Swapchain<'a> {
    pub fn new(ctx: Arc<Context>, window: &'a Window) -> Self {
        let instance = surface::Instance::new(&ctx.entry, &ctx.instance);
        let device = swapchain::Device::new(&ctx.instance, &ctx.device);

        let surface = window.create_surface(&ctx.entry, &ctx.instance);
        let window_size = window.get_current_client_size();
        let info =
            SurfaceInfoForSwapchain::from(&instance, ctx.physical_device, surface, window_size)
                .expect("failed to get info of surface for creating a swapchain");
        let swapchain =
            create_swapchain(&device, surface, &info, None).expect("failed to create a swapchain");

        Self {
            surface,
            resolution: info.resolution,
            format: info.format,
            swapchain,
            ctx,
            window,
        }
    }
}

impl Drop for Swapchain<'_> {
    fn drop(&mut self) {
        unsafe {
            let instance = surface::Instance::new(&self.ctx.entry, &self.ctx.instance);
            let device = swapchain::Device::new(&self.ctx.instance, &self.ctx.device);

            device.destroy_swapchain(self.swapchain, None);
            instance.destroy_surface(self.surface, None);
        }
    }
}

fn create_swapchain(
    device: &swapchain::Device,
    surface: vk::SurfaceKHR,
    info: &SurfaceInfoForSwapchain,
    old_swapchain: Option<vk::SwapchainKHR>,
) -> VkResult<vk::SwapchainKHR> {
    let mut ci = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(info.min_image_count)
        .image_color_space(info.format.color_space)
        .image_format(info.format.format)
        .image_extent(info.resolution)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(info.pre_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(info.present_mode)
        .clipped(true)
        .image_array_layers(1);
    if let Some(old_swapchain) = old_swapchain {
        ci.old_swapchain = old_swapchain;
    }
    unsafe { device.create_swapchain(&ci, None) }
}
