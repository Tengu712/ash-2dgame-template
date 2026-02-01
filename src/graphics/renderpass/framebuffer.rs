use super::super::{context::Context, image::Image, swapchain::Swapchain};
use crate::logs::*;
use ash::{Device, vk};

/// フレームバッファ
///
/// NOTE: レンダーターゲットとなるイメージもここで管理する。
///       特に2Dゲームでは1個のレンダーパスで事足りるため、
///       レンダーターゲットと関連性の高いここに凝集しておく。
///
/// WARN: スワップチェーンより短命であること。
pub struct Framebuffer {
    pub image: Image,
    pub framebuffer: vk::Framebuffer,
    pub clear_colors: Vec<vk::ClearValue>,
}

impl Framebuffer {
    pub fn new(
        ctx: &Context,
        render_pass: vk::RenderPass,
        width: u32,
        height: u32,
        image: Image,
    ) -> Self {
        let framebuffer =
            create_framebuffer(&ctx.device, render_pass, width, height, &[image.view()]);
        let clear_colors = vec![vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];
        Self {
            image,
            framebuffer,
            clear_colors,
        }
    }

    pub fn from_swapchain(
        ctx: &Context,
        render_pass: vk::RenderPass,
        swapchain: &Swapchain,
    ) -> Vec<Self> {
        let width = swapchain.resolution.width;
        let height = swapchain.resolution.height;
        swapchain
            .images
            .iter()
            .copied()
            .map(|image| {
                Image::wrap(
                    ctx,
                    image,
                    swapchain.format.format,
                    vk::ImageAspectFlags::COLOR,
                )
            })
            .map(|image| Self::new(ctx, render_pass, width, height, image))
            .collect()
    }

    pub fn destroy(self, ctx: &Context) {
        unsafe {
            ctx.device.destroy_framebuffer(self.framebuffer, None);
            self.image.destroy(ctx);
        }
    }
}

fn create_framebuffer(
    device: &Device,
    render_pass: vk::RenderPass,
    width: u32,
    height: u32,
    attachments: &[vk::ImageView],
) -> vk::Framebuffer {
    let ci = vk::FramebufferCreateInfo::default()
        .render_pass(render_pass)
        .attachments(attachments)
        .width(width)
        .height(height)
        .layers(1);
    unsafe {
        device
            .create_framebuffer(&ci, None)
            .expect_log("failed to create a framebuffer")
    }
}
