use super::{context::Context, image::wrap::ImageView};
use crate::window::Window;
use ash::{khr::swapchain, prelude::VkResult, vk};
use std::rc::Rc;

mod info;

use info::SurfaceInfoForSwapchain;

/// スワップチェーン
///
/// WARN: コード的には許容されているが、1ウィンドウ1スワップチェーンを保つこと。
pub struct Swapchain {
    pub surface: vk::SurfaceKHR,
    pub resolution: vk::Extent2D,
    pub format: vk::SurfaceFormatKHR,
    pub swapchain: vk::SwapchainKHR,
    pub image_views: Vec<ImageView>,
    ctx: Rc<Context>,
    window: Rc<Window>,
}

impl Swapchain {
    pub fn new(ctx: Rc<Context>, window: Rc<Window>) -> Self {
        let swapchain_loader = ctx.swapchain_loader();

        // surface
        #[cfg(target_os = "windows")]
        let surface = window
            .create_surface(&ctx.win32_surface_loader())
            .expect("failed to create the surface of the window");

        // swapchain
        let window_size = window
            .get_current_client_size()
            .expect("failed to get the client size of the window");
        let info = SurfaceInfoForSwapchain::from(
            &ctx.surface_loader(),
            ctx.physical_device,
            surface,
            window_size,
        )
        .expect("failed to get info of surface for creating a swapchain");
        let swapchain = create_swapchain(&swapchain_loader, surface, &info, None)
            .expect("failed to create a swapchain");

        // image views
        let image_views =
            collect_image_views(&ctx, &swapchain_loader, swapchain, info.format.format)
                .expect("failed to collect the views of swapchain images");

        Self {
            surface,
            resolution: info.resolution,
            format: info.format,
            swapchain,
            image_views,
            ctx,
            window,
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.image_views.clear();
            self.ctx
                .swapchain_loader()
                .destroy_swapchain(self.swapchain, None);
            self.ctx
                .surface_loader()
                .destroy_surface(self.surface, None);
        }
    }
}

fn create_swapchain(
    swapchain_loader: &swapchain::Device,
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
    unsafe { swapchain_loader.create_swapchain(&ci, None) }
}

fn collect_image_views(
    ctx: &Rc<Context>,
    swapchain_loader: &swapchain::Device,
    swapchain: vk::SwapchainKHR,
    format: vk::Format,
) -> VkResult<Vec<ImageView>> {
    unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)?
            .into_iter()
            .map(|image| {
                ImageView::from(Rc::clone(ctx), image, format, vk::ImageAspectFlags::COLOR)
            })
            .collect()
    }
}
