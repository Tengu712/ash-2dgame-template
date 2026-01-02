use super::{context::Context, swapchain::Swapchain};
use ash::{Device, prelude::VkResult, vk};
use std::rc::Rc;

mod framebuffer;
mod pipeline;

use framebuffer::Framebuffer;
use pipeline::Pipeline;

struct FrameObject<'a> {
    framebuffer: Framebuffer<'a>,
    finished_semaphore: vk::Semaphore,
}

/// レンダーコマンドの実行タイミングを制御するためのセマフォ群
///
/// NOTE: スワップチェーンが絡んでこなければ描画完了をフェンスで待機するだけでいい。
///       ので、例えばオフスクリーンレンダリングを行う場合は不要。
///
/// WARN: 専ら`RenderPass::record_render_commands()`の返戻値として使われる。
///       ので、自力作成はしないこと。
///
/// WARN: `RenderPass`より短命であるように管理すること。
///       特に、1フレーム中でのみ生きるようにすること。
pub struct RenderingSemaphores {
    /// スワップチェーンイメージが利用可能になるまで待機するためのセマフォ
    pub started_semaphore: vk::Semaphore,
    /// 描画が完了するまで待機するためのセマフォ
    ///
    /// NOTE: コマンドバッファ提出時にシグナルセマフォに指定すること
    pub finished_smaphore: vk::Semaphore,
}

pub struct RenderPass<'a> {
    pub render_pass: vk::RenderPass,
    pipeline: Pipeline,
    frame_objects: Vec<FrameObject<'a>>,

    /// スワップチェーンイメージが利用可能になるまで待機するためのセマフォ
    /// スワップチェーンイメージの個数ある
    started_semaphores: Vec<vk::Semaphore>,
    /// 上のセマフォを順繰りに取得するためのインデックスカウンタ
    ///
    /// NOTE: 実際に利用可能になるスワップチェーンイメージのインデックスとは無関係。
    started_semaphores_counter: usize,

    swapchain: &'a Swapchain,
    ctx: Rc<Context>,
}

impl<'a> RenderPass<'a> {
    pub fn new(ctx: Rc<Context>, swapchain: &'a Swapchain) -> VkResult<Self> {
        let render_pass = create_render_pass(&ctx.device, swapchain.format.format)?;

        let pipeline = Pipeline::new(Rc::clone(&ctx), render_pass)?;

        let image_count = swapchain.image_views.len();

        let mut frame_objects = Vec::with_capacity(image_count);
        for image_view in &swapchain.image_views {
            let framebuffer = Framebuffer::new(
                Rc::clone(&ctx),
                render_pass,
                swapchain.resolution.width,
                swapchain.resolution.height,
                &image_view.view,
            )?;
            frame_objects.push(FrameObject {
                framebuffer,
                finished_semaphore: create_semaphore(&ctx.device)?,
            });
        }

        let started_semaphores = (0..image_count)
            .map(|_| create_semaphore(&ctx.device))
            .collect::<VkResult<Vec<_>>>()?;

        Ok(Self {
            render_pass,
            pipeline,
            frame_objects,
            started_semaphores,
            started_semaphores_counter: 0,
            swapchain,
            ctx,
        })
    }

    /// レンダーコマンドを記録する関数
    ///
    /// 適切なスワップチェーンイメージを対象とする。
    ///
    /// 対象となったスワップチェーンイメージのインデックスと
    /// コマンドバッファ提出時に有用となるセマフォ群を返す。
    pub fn record_render_commands(
        &mut self,
        command_buffer: vk::CommandBuffer,
    ) -> VkResult<(u32, RenderingSemaphores)> {
        // スワップチェーンイメージ利用可能待機用セマフォ取得
        let started_semaphore = self.started_semaphores[self.started_semaphores_counter];

        // フレームオブジェクト取得
        let index = self.swapchain.acquire_next_image_index(started_semaphore)?;
        let Some(frame_object) = self.frame_objects.get(index as usize) else {
            return Err(vk::Result::ERROR_UNKNOWN);
        };

        // レンダーパス開始
        let bi = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(frame_object.framebuffer.framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.resolution,
            })
            .clear_values(&frame_object.framebuffer.clear_colors);
        unsafe {
            self.ctx
                .device
                .cmd_begin_render_pass(command_buffer, &bi, vk::SubpassContents::INLINE)
        };

        // パイプラインバインド
        unsafe {
            self.ctx.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            )
        };

        // 現在のスワップチェーンイメージの解像度に合わせてビューポート設定
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.swapchain.resolution.width as f32,
            height: self.swapchain.resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: self.swapchain.resolution,
        }];
        unsafe {
            self.ctx
                .device
                .cmd_set_viewport(command_buffer, 0, &viewports)
        };
        unsafe {
            self.ctx
                .device
                .cmd_set_scissor(command_buffer, 0, &scissors)
        };

        // DEBUG:
        unsafe { self.ctx.device.cmd_draw(command_buffer, 4, 1, 0, 0) };

        // レンダーパス終了
        unsafe { self.ctx.device.cmd_end_render_pass(command_buffer) };

        self.started_semaphores_counter =
            (self.started_semaphores_counter + 1) % self.started_semaphores.len();
        Ok((
            index,
            RenderingSemaphores {
                started_semaphore,
                finished_smaphore: frame_object.finished_semaphore,
            },
        ))
    }
}

impl Drop for RenderPass<'_> {
    fn drop(&mut self) {
        unsafe {
            for &semaphore in self.started_semaphores.iter() {
                self.ctx.device.destroy_semaphore(semaphore, None);
            }
            for frame_object in self.frame_objects.iter() {
                self.ctx
                    .device
                    .destroy_semaphore(frame_object.finished_semaphore, None);
            }
            self.frame_objects.clear();
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
