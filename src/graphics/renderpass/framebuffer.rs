use super::super::context::Context;
use ash::{Device, prelude::VkResult, vk};
use std::{marker::PhantomData, rc::Rc};

/// フレームバッファ
///
/// NOTE: レンダーターゲットとなるイメージもここで管理する。
///       特に2Dゲームでは1個のレンダーパスで事足りるため、
///       レンダーターゲットと関連性の高いここに凝集しておく。
pub struct Framebuffer<'a> {
    pub framebuffer: vk::Framebuffer,
    pub clear_colors: Vec<vk::ClearValue>,
    ctx: Rc<Context>,
    _swapchain_image_view: PhantomData<&'a ()>,
}

impl<'a> Framebuffer<'a> {
    pub fn new(
        ctx: Rc<Context>,
        render_pass: vk::RenderPass,
        width: u32,
        height: u32,
        swapchain_image_view: &'a vk::ImageView,
    ) -> VkResult<Self> {
        let attachments = [*swapchain_image_view];
        let framebuffer =
            create_framebuffer(&ctx.device, render_pass, width, height, &attachments)?;

        let clear_colors = vec![vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        Ok(Self {
            framebuffer,
            clear_colors,
            ctx,
            _swapchain_image_view: PhantomData,
        })
    }
}

impl Drop for Framebuffer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_framebuffer(self.framebuffer, None);
        }
    }
}

fn create_framebuffer(
    device: &Device,
    render_pass: vk::RenderPass,
    width: u32,
    height: u32,
    attachments: &[vk::ImageView],
) -> VkResult<vk::Framebuffer> {
    let ci = vk::FramebufferCreateInfo::default()
        .render_pass(render_pass)
        .attachments(attachments)
        .width(width)
        .height(height)
        .layers(1);
    unsafe { device.create_framebuffer(&ci, None) }
}
