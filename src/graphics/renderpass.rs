use super::{context::Context, swapchain::Swapchain};
use crate::logs::*;
use ash::{Device, vk};

pub mod framebuffer;
pub mod pipeline;

use framebuffer::Framebuffer;
use pipeline::Pipeline;

#[derive(Debug, Default, Clone, Copy)]
pub struct RenderAreas {
    /// クリアの対象となる範囲
    ///
    /// WARN: `vireport`よりも広いこと。
    /// NOTE: 取り敢えずレンダーターゲット全体を指定すると良い。
    pub render_area: vk::Rect2D,
    /// NDCのマッピング先範囲
    pub viewport: vk::Viewport,
    /// 描画結果を残す範囲
    pub scissor: vk::Rect2D,
}

pub struct RenderPass {
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<Framebuffer>,
    pub pipeline: Pipeline,
}

impl RenderPass {
    /// コンストラクタ
    ///
    /// * ctx - コンテキスト
    /// * format - スワップチェーンイメージのフォーマット
    /// * set_layouts - ディスクリプタセット
    pub fn new(
        ctx: &Context,
        swapchain: &Swapchain,
        set_layouts: &[vk::DescriptorSetLayout],
    ) -> Self {
        let render_pass = create_render_pass(&ctx.device, swapchain.format.format);
        let framebuffers = Framebuffer::from_swapchain(ctx, render_pass, swapchain);
        let pipeline = Pipeline::new(ctx, render_pass, set_layouts);
        Self {
            render_pass,
            framebuffers,
            pipeline,
        }
    }

    pub fn destroy(self, ctx: &Context) {
        unsafe {
            self.pipeline.destroy(ctx);
            for framebuffer in self.framebuffers {
                framebuffer.destroy(ctx);
            }
            ctx.device.destroy_render_pass(self.render_pass, None);
        }
    }
}

impl RenderPass {
    pub fn recreate_framebuffers(self, ctx: &Context, swapchain: &Swapchain) -> Self {
        for framebuffer in self.framebuffers {
            framebuffer.destroy(ctx);
        }
        Self {
            framebuffers: Framebuffer::from_swapchain(ctx, self.render_pass, swapchain),
            ..self
        }
    }
}

impl RenderPass {
    pub fn record_render_commands(
        &self,
        ctx: &Context,
        command_buffer: vk::CommandBuffer,
        areas: RenderAreas,
        count: usize,
        index: u32,
    ) {
        unsafe {
            let framebuffer = &self.framebuffers[index as usize];

            // レンダーパス開始
            let bi = vk::RenderPassBeginInfo::default()
                .render_pass(self.render_pass)
                .framebuffer(framebuffer.framebuffer)
                .render_area(areas.render_area)
                .clear_values(&framebuffer.clear_colors);
            ctx.device
                .cmd_begin_render_pass(command_buffer, &bi, vk::SubpassContents::INLINE);

            // パイプラインバインド
            ctx.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );

            // ビューポート設定
            ctx.device
                .cmd_set_viewport(command_buffer, 0, &[areas.viewport]);
            ctx.device
                .cmd_set_scissor(command_buffer, 0, &[areas.scissor]);

            // ドローコール
            // 2Dゲームなので1ドローコールで済む
            ctx.device.cmd_draw(command_buffer, 4, count as u32, 0, 0);

            // レンダーパス終了
            ctx.device.cmd_end_render_pass(command_buffer);
        }
    }
}

/// レンダーパスを作成する関数
///
/// - subpass 0:
///   color:
///     - attachment 0
fn create_render_pass(device: &Device, format: vk::Format) -> vk::RenderPass {
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
    unsafe {
        device
            .create_render_pass(&ci, None)
            .expect_log("failed to create a render pass")
    }
}
