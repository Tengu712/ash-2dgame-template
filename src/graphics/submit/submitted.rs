use super::Submitter;
use ash::prelude::VkResult;

/// 提出済みのコマンドバッファ
///
/// NOTE: 明示的に`wait()`によって実行完了を待機しない場合、
///       drop時にコマンドの実行完了が待機される。
//
// NOTE: `Submitter`を可変参照で持つことで、
//       `SubmittedCommandBuffer`がdropされるまで`Submitter`を操作できなくする。
pub struct SubmittedCommandBuffer<'a>(&'a mut Submitter, bool);

impl<'a> SubmittedCommandBuffer<'a> {
    pub(super) fn new(submitter: &'a mut Submitter) -> Self {
        Self(submitter, false)
    }

    pub fn wait(mut self) -> VkResult<()> {
        unsafe {
            self.1 = true;
            self.0
                .ctx
                .device
                .wait_for_fences(&[self.0.fence], true, u64::MAX)
        }
    }
}

impl Drop for SubmittedCommandBuffer<'_> {
    fn drop(&mut self) {
        unsafe {
            if !self.1 {
                let _ = self
                    .0
                    .ctx
                    .device
                    .wait_for_fences(&[self.0.fence], true, u64::MAX);
            }
        }
    }
}
