use super::context::Context;
use crate::logs::*;
use ash::vk;
use std::{marker::PhantomData, mem, slice};

macro_rules! define_submitter_state {
    ($name: ident) => {
        #[repr(C)]
        pub struct $name {
            command_pool: vk::CommandPool,
            command_buffer: vk::CommandBuffer,
            fence: vk::Fence,
            _not_send_sync: PhantomData<*const ()>,
        }
    };
}

define_submitter_state!(Submitter);
define_submitter_state!(RecordingSubmitter);
define_submitter_state!(SubmittedSubmitter);

impl Submitter {
    pub fn new(ctx: &Context) -> Self {
        unsafe {
            let ci =
                vk::CommandPoolCreateInfo::default().queue_family_index(ctx.queue_family_index);
            let command_pool = ctx
                .device
                .create_command_pool(&ci, None)
                .expect_log("failed to create a command pool");

            let ai = vk::CommandBufferAllocateInfo::default()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            let command_buffer = ctx
                .device
                .allocate_command_buffers(&ai)
                .expect_log("failed to allocate a command buffer")[0];

            let ci = vk::FenceCreateInfo::default();
            let fence = ctx
                .device
                .create_fence(&ci, None)
                .expect_log("failed to create a fence");

            Self {
                command_pool,
                command_buffer,
                fence,
                _not_send_sync: PhantomData,
            }
        }
    }

    pub fn destroy(self, ctx: &Context) {
        unsafe {
            let _ = ctx
                .device
                .reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty());
            ctx.device
                .free_command_buffers(self.command_pool, &[self.command_buffer]);
            ctx.device.destroy_command_pool(self.command_pool, None);
            ctx.device.destroy_fence(self.fence, None);
        }
    }
}

impl Submitter {
    pub fn start(self, ctx: &Context) -> RecordingSubmitter {
        unsafe {
            ctx.device
                .reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())
                .expect_log("failed to reset a command pool");
            let bi = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            ctx.device
                .begin_command_buffer(self.command_buffer, &bi)
                .expect_log("failed to begin a command buffer");
            mem::transmute(self)
        }
    }
}

impl RecordingSubmitter {
    /// コマンドバッファを取得する関数
    ///
    /// WARN: この`command_buffer`はコマンドの記録に対してのみ用いること。
    ///       決して、リセット・開始・終了等したりせず、
    ///       またコピーやクローンをしないこと。
    pub fn command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    /// コマンドを提出する関数
    ///
    /// * wait_infos - 提出までに待機するセマフォおよびパイプラインステージ
    /// * signal_semaphores - 提出されたコマンドの実行完了時にシグナルするセマフォ
    pub fn submit(
        self,
        ctx: &Context,
        wait_infos: &[(vk::Semaphore, vk::PipelineStageFlags)],
        signal_semaphores: &[vk::Semaphore],
    ) -> SubmittedSubmitter {
        unsafe {
            ctx.device
                .end_command_buffer(self.command_buffer)
                .expect_log("failed to end a command buffer");

            ctx.device
                .reset_fences(&[self.fence])
                .expect_log("failed to reset a fence");

            let (wait_semaphores, wait_dst_stage_masks): (Vec<_>, Vec<_>) =
                wait_infos.iter().copied().unzip();
            let si = vk::SubmitInfo::default()
                .command_buffers(slice::from_ref(&self.command_buffer))
                .wait_dst_stage_mask(&wait_dst_stage_masks)
                .wait_semaphores(&wait_semaphores)
                .signal_semaphores(signal_semaphores);
            ctx.device
                .queue_submit(ctx.queue, &[si], self.fence)
                .expect_log("failed to submit a command buffer");

            mem::transmute(self)
        }
    }
}

impl SubmittedSubmitter {
    pub fn wait(self, ctx: &Context) -> Submitter {
        unsafe {
            ctx.device
                .wait_for_fences(&[self.fence], true, u64::MAX)
                .expect_log("failed to wait a fence");
            mem::transmute(self)
        }
    }
}
