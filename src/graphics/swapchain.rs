use super::context::Context;
use crate::{logs::*, window::Window};
use ash::vk;
use std::slice;

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
    pub images: Vec<vk::Image>,
}

impl Swapchain {
    pub fn new(window: &Window, ctx: &Context) -> Self {
        #[cfg(target_os = "windows")]
        let surface = window.create_surface(&ctx.win32_surface_loader);
        #[cfg(target_os = "macos")]
        let surface = window.create_surface(&ctx.metal_surface_loader);
        #[cfg(target_os = "linux")]
        let surface = window.create_surface(&ctx.xcb_surface_loader);
        let surface = surface.expect_log("failed to create a surface");
        let (resolution, format, swapchain, images) =
            create_swapchain_util(window, ctx, surface, None);
        Self {
            surface,
            resolution,
            format,
            swapchain,
            images,
        }
    }

    pub fn destroy(self, ctx: &Context, keep_surface: bool) {
        unsafe {
            ctx.swapchain_loader.destroy_swapchain(self.swapchain, None);
            if !keep_surface {
                ctx.surface_loader.destroy_surface(self.surface, None);
            }
        }
    }

    pub fn recreate(self, window: &Window, ctx: &Context) -> Self {
        let surface = self.surface;
        let (resolution, format, swapchain, images) =
            create_swapchain_util(window, ctx, surface, Some(self.swapchain));
        self.destroy(ctx, true);
        Self {
            surface,
            resolution,
            format,
            swapchain,
            images,
        }
    }
}

impl Swapchain {
    /// 描画準備が完了したスワップチェーンイメージのインデックスを取得する関数
    ///
    /// そのスワップチェーンイメージの準備が完了すると`signal_semaphore`がシグナルされる。
    ///
    /// Swapchainを再作成すべき場合は`Err`を返し、そうでなければ`Ok`を返す。
    pub fn acquire_next_image_index(
        &self,
        ctx: &Context,
        signal_semaphore: vk::Semaphore,
    ) -> Result<u32, ()> {
        let res = unsafe {
            ctx.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                signal_semaphore,
                vk::Fence::null(),
            )
        };
        match res {
            Ok((index, false)) => Ok(index),
            Ok((_, true)) => Err(()),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err(()),
            _ => panic_log("failed to queue a presentation command"),
        }
    }

    /// プレゼンテーションコマンドをキューする関数
    ///
    /// `wait_semaphores`がシグナルされると、
    /// `index`番目のスワップチェーンイメージがプレゼントされる。
    ///
    /// Swapchainを再作成すべき場合は`Err`を返し、そうでなければ`Ok`を返す。
    pub fn queue_presentation_command(
        &self,
        ctx: &Context,
        index: u32,
        wait_semaphore: vk::Semaphore,
    ) -> Result<(), ()> {
        let pi = vk::PresentInfoKHR::default()
            .wait_semaphores(slice::from_ref(&wait_semaphore))
            .swapchains(slice::from_ref(&self.swapchain))
            .image_indices(slice::from_ref(&index));
        let res = unsafe { ctx.swapchain_loader.queue_present(ctx.queue, &pi) };
        match res {
            Ok(false) => Ok(()),
            Ok(true) => Err(()),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err(()),
            _ => panic_log("failed to queue a presentation command"),
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

fn create_swapchain_util(
    window: &Window,
    ctx: &Context,
    surface: vk::SurfaceKHR,
    old_swapchain: Option<vk::SwapchainKHR>,
) -> (
    vk::Extent2D,
    vk::SurfaceFormatKHR,
    vk::SwapchainKHR,
    Vec<vk::Image>,
) {
    let window_size = window
        .get_current_client_size()
        .expect_log("failed to get the client size of the window");
    let info = SurfaceInfoForSwapchain::from(ctx, surface, window_size);

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
    let swapchain = unsafe {
        ctx.swapchain_loader
            .create_swapchain(&ci, None)
            .expect_log("failed to create a swapchain")
    };

    let images = unsafe {
        ctx.swapchain_loader
            .get_swapchain_images(swapchain)
            .expect_log("failed to get swapchain images")
    };

    (info.resolution, info.format, swapchain, images)
}
