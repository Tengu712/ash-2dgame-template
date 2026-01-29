use super::{context::Context, descriptor::Descriptors, framebuffer::Framebuffer};
use ash::{Device, prelude::VkResult, vk};
use std::{marker::PhantomData, rc::Rc};

pub mod pipeline;

use pipeline::Pipeline;

#[derive(Clone, Copy)]
pub struct Semaphores {
    /// スワップチェーンイメージが利用可能になるまで待機するためのセマフォ
    ///
    /// NOTE: スワップチェーンイメージのインデックス取得時にシグナルセマフォに指定すること。
    pub started_semaphore: vk::Semaphore,
    /// 描画が完了するまで待機するためのセマフォ
    ///
    /// NOTE: コマンドバッファ提出時にシグナルセマフォに指定すること。
    pub finished_semaphore: vk::Semaphore,
}

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

pub struct RenderPass<'a> {
    pub render_pass: vk::RenderPass,
    pub pipeline: Pipeline,

    /// セマフォを順繰りに取得するためのインデックスカウンタ
    ///
    /// NOTE: 実際に利用可能になるスワップチェーンイメージのインデックスとは無関係。
    semaphores_counter: usize,
    semaphores: Vec<Semaphores>,

    ctx: Rc<Context>,

    _descriptor: PhantomData<&'a ()>,
}

impl<'a> RenderPass<'a> {
    /// コンストラクタ
    ///
    /// * ctx - コンテキスト
    /// * image_count - スワップチェーンイメージの個数
    /// * format - スワップチェーンイメージのフォーマット
    pub fn new(
        ctx: Rc<Context>,
        image_count: usize,
        format: vk::Format,
        descriptors: &'a Descriptors,
    ) -> VkResult<Self> {
        let render_pass = create_render_pass(&ctx.device, format)?;
        let pipeline = Pipeline::new(
            Rc::clone(&ctx),
            render_pass,
            &descriptors.collect_set_layouts(),
        )?;
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
            _descriptor: PhantomData,
        })
    }

    /// 現在のフレームのセマフォを取得する関数
    ///
    /// CAUTION: 必ず`record_render_commands()`よりも前に呼び出し、かつ後に呼び出さないこと。
    /// CAUTION: フレームをまたいで使いまわさないこと。
    pub fn semaphores(&mut self) -> Semaphores {
        self.semaphores_counter = (self.semaphores_counter + 1) % self.semaphores.len();
        self.semaphores[self.semaphores_counter]
    }

    pub fn record_render_commands(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer: &Framebuffer,
        areas: RenderAreas,
        count: usize,
    ) -> VkResult<()> {
        unsafe {
            // レンダーパス開始
            let bi = vk::RenderPassBeginInfo::default()
                .render_pass(self.render_pass)
                .framebuffer(framebuffer.framebuffer)
                .render_area(areas.render_area)
                .clear_values(&framebuffer.clear_colors);
            self.ctx
                .device
                .cmd_begin_render_pass(command_buffer, &bi, vk::SubpassContents::INLINE);

            // パイプラインバインド
            self.ctx.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );

            // ビューポート設定
            self.ctx
                .device
                .cmd_set_viewport(command_buffer, 0, &[areas.viewport]);
            self.ctx
                .device
                .cmd_set_scissor(command_buffer, 0, &[areas.scissor]);

            // ドローコール
            // 2Dゲームなので1ドローコールで済む
            self.ctx
                .device
                .cmd_draw(command_buffer, 4, count as u32, 0, 0);

            // レンダーパス終了
            self.ctx.device.cmd_end_render_pass(command_buffer);

            Ok(())
        }
    }

    pub fn recreate_semaohores(&mut self) -> VkResult<()> {
        unsafe {
            for semaphore in &mut self.semaphores {
                self.ctx
                    .device
                    .destroy_semaphore(semaphore.started_semaphore, None);
                self.ctx
                    .device
                    .destroy_semaphore(semaphore.finished_semaphore, None);
                *semaphore = Semaphores {
                    started_semaphore: create_semaphore(&self.ctx.device)?,
                    finished_semaphore: create_semaphore(&self.ctx.device)?,
                };
            }
            self.semaphores_counter = 0;
            Ok(())
        }
    }
}

impl Drop for RenderPass<'_> {
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
