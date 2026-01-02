use super::context::Context;
use ash::{Device, prelude::VkResult, vk};
use std::rc::Rc;

mod pipeline;
pub mod recording;

use pipeline::Pipeline;
use recording::RecordingRenderPass;

#[derive(Clone, Copy)]
pub struct Semaphores {
    /// スワップチェーンイメージが利用可能になるまで待機するためのセマフォ
    ///
    /// NOTE: スワップチェーンイメージのインデックス取得時にシグナルセマフォに指定すること
    pub started_semaphore: vk::Semaphore,
    /// 描画が完了するまで待機するためのセマフォ
    ///
    /// NOTE: コマンドバッファ提出時にシグナルセマフォに指定すること
    pub finished_semaphore: vk::Semaphore,
}

pub struct RenderPass {
    pub render_pass: vk::RenderPass,
    pipeline: Pipeline,
    semaphores: Vec<Semaphores>,

    /// セマフォを順繰りに取得するためのインデックスカウンタ
    ///
    /// NOTE: 実際に利用可能になるスワップチェーンイメージのインデックスとは無関係。
    semaphores_counter: usize,

    ctx: Rc<Context>,
}

impl RenderPass {
    /// コンストラクタ
    ///
    /// * ctx - コンテキスト
    /// * image_count - スワップチェーンイメージの個数
    /// * format - スワップチェーンイメージのフォーマット
    pub fn new(ctx: Rc<Context>, image_count: usize, format: vk::Format) -> VkResult<Self> {
        let render_pass = create_render_pass(&ctx.device, format)?;
        let pipeline = Pipeline::new(Rc::clone(&ctx), render_pass)?;
        let mut semaphores = Vec::with_capacity(image_count);
        for _ in 0..image_count {
            semaphores.push(Semaphores {
                started_semaphore: create_semaphore(&ctx.device)?,
                finished_semaphore: create_semaphore(&ctx.device)?,
            });
        }
        Ok(Self {
            render_pass,
            pipeline,
            semaphores,
            semaphores_counter: 0,
            ctx,
        })
    }

    pub fn prepare<'a>(&'a mut self) -> RecordingRenderPass<'a> {
        RecordingRenderPass(self)
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            for semaphore in self.semaphores.iter() {
                self.ctx
                    .device
                    .destroy_semaphore(semaphore.finished_semaphore, None);
                self.ctx
                    .device
                    .destroy_semaphore(semaphore.started_semaphore, None);
            }
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

fn create_semaphore(device: &Device) -> VkResult<vk::Semaphore> {
    unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }
}
