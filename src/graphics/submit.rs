use super::context::Context;
use ash::{Device, prelude::VkResult, vk};
use std::{marker::PhantomData, sync::Arc};

pub mod recording;
pub mod submitted;

use recording::RecordingCommandBuffer;

/// Vulkanコマンドを記録・提出するためのオブジェクト
///
/// 単一のスレッド上で動作する。
/// 複数スレッドからコマンドを提出する場合は各スレッドでSubmitterを作成すること。
pub struct Submitter {
    ctx: Arc<Context>,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,

    // NOTE: 複数スレッドでの共有を防ぐため。
    // NOTE: 生ポインタはSendでもSyncでもない。
    _not_send_sync: PhantomData<*const ()>,
}

impl Submitter {
    pub fn new(ctx: Arc<Context>) -> Self {
        let command_pool = create_command_pool(&ctx.device, ctx.queue_family_index);
        let command_buffer = allocate_command_buffer(&ctx.device, command_pool);
        let fence = create_fence(&ctx.device);
        Self {
            ctx,
            command_pool,
            command_buffer,
            fence,
            _not_send_sync: PhantomData,
        }
    }

    pub fn prepare<'a>(&'a self) -> VkResult<RecordingCommandBuffer<'a>> {
        unsafe {
            self.ctx
                .device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())?;
            RecordingCommandBuffer::new(self)
        }
    }
}

impl Drop for Submitter {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .ctx
                .device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty());
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

/// コマンドプールを作成する関数
///
/// このコマンドプールから割り当てられたコマンドバッファは個別にリセットできる。
fn create_command_pool(device: &Device, queue_family_index: u32) -> vk::CommandPool {
    let ci = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family_index);
    unsafe {
        device
            .create_command_pool(&ci, None)
            .expect("failed to create a command pool")
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
            .expect("failed to allocate a command buffer for a submitter")[0]
    }
}

fn create_fence(device: &Device) -> vk::Fence {
    let ci = vk::FenceCreateInfo::default();
    unsafe {
        device
            .create_fence(&ci, None)
            .expect("failed to create a fence for a submitter")
    }
}
