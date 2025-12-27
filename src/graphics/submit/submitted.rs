use super::super::Context;
use ash::{prelude::VkResult, vk};
use std::marker::PhantomData;

/// 提出済みのコマンドバッファ
///
/// 単一のスレッド上で動作する。
///
/// NOTE: 明示的に`wait()`によって実行完了を待機しない場合、
///       drop時にコマンドの実行完了が待機される。
pub struct SubmittedCommandBuffer<'a> {
    ctx: &'a Context,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,
    waited: bool,

    // NOTE: Submitterと同様。
    _not_send_sync: PhantomData<*const ()>,
}

impl<'a> SubmittedCommandBuffer<'a> {
    pub(super) fn new(
        ctx: &'a Context,
        command_buffer: vk::CommandBuffer,
        fence: vk::Fence,
    ) -> Self {
        Self {
            ctx,
            command_buffer,
            fence,
            waited: false,
            _not_send_sync: PhantomData,
        }
    }

    pub fn wait(mut self) -> VkResult<()> {
        unsafe {
            self.waited = true;
            self.ctx
                .device
                .wait_for_fences(&[self.fence], true, u64::MAX)
        }
    }
}

impl Drop for SubmittedCommandBuffer<'_> {
    fn drop(&mut self) {
        unsafe {
            if !self.waited {
                let _ = self
                    .ctx
                    .device
                    .wait_for_fences(&[self.fence], true, u64::MAX);
            }
        }
    }
}
