use super::{context::Context, image::wrap::ImageView};
use crate::{logs::*, window::Window};
use ash::{khr::swapchain, prelude::VkResult, vk};
use std::{rc::Rc, slice};

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

    /// 再作成時に`surface`を破棄しないようにするためのフラグ
    keep_surface: bool,
}

impl Swapchain {
    pub fn new(ctx: Rc<Context>, window: Rc<Window>) -> Self {
        // surface
        #[cfg(target_os = "windows")]
        let surface = window.create_surface(&ctx.win32_surface_loader);
        #[cfg(target_os = "macos")]
        let surface = window.create_surface(&ctx.metal_surface_loader);
        #[cfg(target_os = "linux")]
        let surface = window.create_surface(&ctx.xcb_surface_loader);

        let surface = surface.expect_log("failed to create the surface of the window");

        // swapchain
        let window_size = window
            .get_current_client_size()
            .expect_log("failed to get the client size of the window");
        let info = SurfaceInfoForSwapchain::from(
            &ctx.surface_loader,
            ctx.physical_device,
            surface,
            window_size,
        )
        .expect_log("failed to get info of surface for creating a swapchain");
        let swapchain = create_swapchain(&ctx.swapchain_loader, surface, &info, None)
            .expect_log("failed to create a swapchain");

        // image views
        let image_views =
            collect_image_views(&ctx, &ctx.swapchain_loader, swapchain, info.format.format)
                .expect_log("failed to collect the views of swapchain images");

        Self {
            surface,
            resolution: info.resolution,
            format: info.format,
            swapchain,
            image_views,
            ctx,
            window,
            keep_surface: false,
        }
    }

    pub fn recreate(mut self) -> VkResult<Self> {
        let window_size = self
            .window
            .get_current_client_size()
            .ok_or(vk::Result::ERROR_UNKNOWN)?;
        let info = SurfaceInfoForSwapchain::from(
            &self.ctx.surface_loader,
            self.ctx.physical_device,
            self.surface,
            window_size,
        )?;
        let swapchain = create_swapchain(
            &self.ctx.swapchain_loader,
            self.surface,
            &info,
            Some(self.swapchain),
        )?;

        let image_views = collect_image_views(
            &self.ctx,
            &self.ctx.swapchain_loader,
            swapchain,
            info.format.format,
        )?;

        self.keep_surface = true;

        Ok(Self {
            surface: self.surface,
            resolution: info.resolution,
            format: info.format,
            swapchain,
            image_views,
            ctx: Rc::clone(&self.ctx),
            window: Rc::clone(&self.window),
            keep_surface: false,
        })
    }

    pub fn acquire_next_image_index(&self, signal_semaphore: vk::Semaphore) -> VkResult<u32> {
        let (index, suboptimal) = unsafe {
            self.ctx.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                signal_semaphore,
                vk::Fence::null(),
            )?
        };
        if suboptimal {
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR)
        } else {
            Ok(index)
        }
    }

    pub fn queue_presentation_command(
        &self,
        index: u32,
        wait_semaphore: vk::Semaphore,
    ) -> VkResult<()> {
        let pi = vk::PresentInfoKHR::default()
            .wait_semaphores(slice::from_ref(&wait_semaphore))
            .swapchains(slice::from_ref(&self.swapchain))
            .image_indices(slice::from_ref(&index));
        let suboptimal = unsafe {
            self.ctx
                .swapchain_loader
                .queue_present(self.ctx.queue, &pi)?
        };
        if suboptimal {
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR)
        } else {
            Ok(())
        }
    }
}

impl Swapchain {
    /// アスペクト比の正しいビューポートを計算する関数
    ///
    /// * ratio - アスペクト比 (幅/高さ)
    ///
    /// このビューポートによりNDCがアスペクト比を保って画面中央にマッピングされる。
    pub fn calc_aspect_corrected_viewport(&self, ratio: f32) -> vk::Viewport {
        let swapchain_ratio = self.resolution.width as f32 / self.resolution.height as f32;
        if swapchain_ratio > ratio {
            let h = self.resolution.height as f32;
            let w = h * ratio;
            let x = (self.resolution.width as f32 - w) / 2.0;
            vk::Viewport {
                x,
                y: 0.0,
                width: w,
                height: h,
                min_depth: 0.0,
                max_depth: 1.0,
            }
        } else {
            let w = self.resolution.width as f32;
            let h = w / ratio;
            let y = (self.resolution.height as f32 - h) / 2.0;
            vk::Viewport {
                x: 0.0,
                y,
                width: w,
                height: h,
                min_depth: 0.0,
                max_depth: 1.0,
            }
        }
    }

    /// 現在のスワップチェーンイメージ全体の範囲を取得する関数
    pub fn get_full_rect(&self) -> vk::Rect2D {
        vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: self.resolution,
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.image_views.clear();
            self.ctx
                .swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            if !self.keep_surface {
                self.ctx.surface_loader.destroy_surface(self.surface, None);
            }
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
