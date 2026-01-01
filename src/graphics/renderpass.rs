use super::{context::Context, swapchain::Swapchain};
use ash::{Device, prelude::VkResult, vk};
use std::rc::Rc;

mod framebuffer;

use framebuffer::Framebuffer;

pub struct RenderPass<'a> {
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<Framebuffer<'a>>,
    ctx: Rc<Context>,
}

impl<'a> RenderPass<'a> {
    pub fn new(ctx: Rc<Context>, swapchain: &'a Swapchain) -> VkResult<Self> {
        let render_pass = create_render_pass(&ctx.device, swapchain.format.format)?;
        let mut framebuffers = Vec::with_capacity(swapchain.image_views.len());
        for image_view in &swapchain.image_views {
            framebuffers.push(Framebuffer::new(
                Rc::clone(&ctx),
                render_pass,
                swapchain.resolution.width,
                swapchain.resolution.height,
                &image_view.view,
            )?);
        }
        Ok(Self {
            render_pass,
            framebuffers,
            ctx,
        })
    }
}

impl Drop for RenderPass<'_> {
    fn drop(&mut self) {
        unsafe {
            self.framebuffers.clear();
            self.ctx.device.destroy_render_pass(self.render_pass, None);
        }
    }
}

/// レンダーパスを作成する関数
///
/// - subpass 0:
///   color:
///     - attachment 0
fn create_render_pass(device: &Device, format: vk::Format) -> VkResult<vk::RenderPass> {
    let attachments = [vk::AttachmentDescription {
        format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    }];
    let color_attachment_refs = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];
    let subpasses = [vk::SubpassDescription::default()
        .color_attachments(&color_attachment_refs)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)];
    let ci = vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(&subpasses);
    unsafe { device.create_render_pass(&ci, None) }
}
