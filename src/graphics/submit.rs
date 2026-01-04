use super::context::Context;
use crate::logs::*;
use ash::{Device, prelude::VkResult, vk};
use std::{marker::PhantomData, rc::Rc};

pub mod recording;

use recording::RecordingCommandBuffer;

/// Vulkanコマンドを記録・提出するためのオブジェクト
///
/// 単一のスレッド上で動作する。
pub struct Submitter {
    ctx: Rc<Context>,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,
    running: bool,
    _not_send_sync: PhantomData<*const ()>,
}

impl Submitter {
    pub fn new(ctx: Rc<Context>) -> Self {
        let command_pool = create_command_pool(&ctx.device, ctx.queue_family_index);
        let command_buffer = allocate_command_buffer(&ctx.device, command_pool);
        let fence = create_fence(&ctx.device);
        Self {
            ctx,
            command_pool,
            command_buffer,
            fence,
            running: false,
            _not_send_sync: PhantomData,
        }
    }

    /// コマンド記録を開始する関数
    ///
    /// 前回コマンドを提出した後、`wait()`を呼び出さなかった場合、
    /// 本関数の最初に`wait()`が呼び出される。
    pub fn prepare<'a>(&'a mut self) -> VkResult<RecordingCommandBuffer<'a>> {
        unsafe {
            self.wait()?;
            self.ctx
                .device
                .reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())?;
            RecordingCommandBuffer::new(self)
        }
    }

    /// 前回提出されたコマンドの実行完了を待機する関数
    ///
    /// コマンドが提出されていない場合、スキップされる。
    #[allow(dead_code)]
    pub fn wait(&mut self) -> VkResult<()> {
        unsafe {
            if self.running {
                self.running = false;
                self.ctx
                    .device
                    .wait_for_fences(&[self.fence], true, u64::MAX)
            } else {
                Ok(())
            }
        }
    }
}

impl Drop for Submitter {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .ctx
                .device
                .reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty());
            self.ctx
                .device
                .free_command_buffers(self.command_pool, &[self.command_buffer]);
            self.ctx
                .device
                .destroy_command_pool(self.command_pool, None);
            self.ctx.device.destroy_fence(self.fence, None);
        }
    }
}

fn create_command_pool(device: &Device, queue_family_index: u32) -> vk::CommandPool {
    let ci = vk::CommandPoolCreateInfo::default().queue_family_index(queue_family_index);
    unsafe {
        device
            .create_command_pool(&ci, None)
            .expect_log("failed to create a command pool")
    }
}

fn allocate_command_buffer(device: &Device, command_pool: vk::CommandPool) -> vk::CommandBuffer {
    let ai = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    unsafe {
        device
            .allocate_command_buffers(&ai)
            .expect_log("failed to allocate a command buffer for a submitter")[0]
    }
}

fn create_fence(device: &Device) -> vk::Fence {
    let ci = vk::FenceCreateInfo::default();
    unsafe {
        device
            .create_fence(&ci, None)
            .expect_log("failed to create a fence for a submitter")
    }
}
