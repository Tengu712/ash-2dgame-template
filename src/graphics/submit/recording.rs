use super::{super::Context, Submitter, submitted::SubmittedCommandBuffer};
use ash::{prelude::VkResult, vk};
use std::{marker::PhantomData, slice};

/// 記録中のコマンドバッファ
///
/// 単一のスレッド上で動作する。
///
/// WARN: この`command_buffer`はコマンドの記録に対してのみ用いること。
///       決して、リセット・開始・終了等したりせず、
///       また`RecordingCommandBuffer`をdropしてなお使い続けることはしないこと。
pub struct RecordingCommandBuffer<'a> {
    pub command_buffer: vk::CommandBuffer,
    ctx: &'a Context,
    fence: vk::Fence,

    // NOTE: Submitterと同様。
    _not_send_sync: PhantomData<*const ()>,
}

impl<'a> RecordingCommandBuffer<'a> {
    pub(super) fn new(submitter: &'a Submitter) -> VkResult<Self> {
        unsafe {
            let bi = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            submitter
                .ctx
                .device
                .begin_command_buffer(submitter.command_buffer, &bi)?;
            Ok(Self {
                command_buffer: submitter.command_buffer,
                ctx: &submitter.ctx,
                fence: submitter.fence,
                _not_send_sync: PhantomData,
            })
        }
    }

    /// コマンドを提出する関数
    ///
    /// * wait_infos - 提出までに待機するセマフォおよびパイプラインステージ
    /// * signal_semaphores - 提出されたコマンドの実行完了時にシグナルするセマフォ
    pub fn submit(
        self,
        wait_infos: &[(vk::Semaphore, vk::PipelineStageFlags)],
        signal_semaphores: &[vk::Semaphore],
    ) -> VkResult<SubmittedCommandBuffer<'a>> {
        unsafe {
            self.ctx.device.end_command_buffer(self.command_buffer)?;

            self.ctx.device.reset_fences(&[self.fence])?;

            let queue = self
                .ctx
                .queue
                .lock()
                .map_err(|_| vk::Result::ERROR_UNKNOWN)?;
            let (wait_semaphores, wait_dst_stage_masks): (Vec<_>, Vec<_>) =
                wait_infos.iter().copied().unzip();
            let si = vk::SubmitInfo::default()
                .command_buffers(slice::from_ref(&self.command_buffer))
                .wait_dst_stage_mask(&wait_dst_stage_masks)
                .wait_semaphores(&wait_semaphores)
                .signal_semaphores(signal_semaphores);
            self.ctx.device.queue_submit(*queue, &[si], self.fence)?;

            Ok(SubmittedCommandBuffer::new(
                self.ctx,
                self.command_buffer,
                self.fence,
            ))
        }
    }
}
