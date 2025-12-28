use super::{Submitter, submitted::SubmittedCommandBuffer};
use ash::{prelude::VkResult, vk};
use std::slice;

/// 記録中のコマンドバッファ
pub struct RecordingCommandBuffer<'a>(&'a mut Submitter);

impl<'a> RecordingCommandBuffer<'a> {
    pub(super) fn new(submitter: &'a mut Submitter) -> VkResult<Self> {
        unsafe {
            let bi = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            submitter
                .ctx
                .device
                .begin_command_buffer(submitter.command_buffer, &bi)?;
            Ok(Self(submitter))
        }
    }

    /// コマンドバッファを取得する関数
    ///
    /// WARN: この`command_buffer`はコマンドの記録に対してのみ用いること。
    ///       決して、リセット・開始・終了等したりせず、
    ///       また`RecordingCommandBuffer`をdropしてなお使い続けることはしないこと。
    pub fn command_buffer(&self) -> vk::CommandBuffer {
        self.0.command_buffer
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
            self.0
                .ctx
                .device
                .end_command_buffer(self.0.command_buffer)?;

            self.0.ctx.device.reset_fences(&[self.0.fence])?;

            let queue = self
                .0
                .ctx
                .queue
                .lock()
                .map_err(|_| vk::Result::ERROR_UNKNOWN)?;
            let (wait_semaphores, wait_dst_stage_masks): (Vec<_>, Vec<_>) =
                wait_infos.iter().copied().unzip();
            let si = vk::SubmitInfo::default()
                .command_buffers(slice::from_ref(&self.0.command_buffer))
                .wait_dst_stage_mask(&wait_dst_stage_masks)
                .wait_semaphores(&wait_semaphores)
                .signal_semaphores(signal_semaphores);
            self.0
                .ctx
                .device
                .queue_submit(*queue, &[si], self.0.fence)?;
            drop(queue);

            Ok(SubmittedCommandBuffer::new(self.0))
        }
    }
}
