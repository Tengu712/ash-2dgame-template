use super::context::Context;
use crate::logs::*;
use ash::vk;

mod array;
mod single;

pub use array::ArrayBuffer;
pub use single::Buffer;

fn create_buffer_util(
    ctx: &Context,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
) -> (vk::Buffer, vk::DeviceMemory) {
    unsafe {
        let ci = vk::BufferCreateInfo::default().size(size).usage(usage);
        let buffer = ctx
            .device
            .create_buffer(&ci, None)
            .expect_log("failed to create a buffer");

        let reqs = ctx.device.get_buffer_memory_requirements(buffer);
        let memory_type_index = ctx
            .find_memory_type_index(
                reqs.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect_log("failed to find a memory type for creating a buffer");
        let ai = vk::MemoryAllocateInfo::default()
            .allocation_size(reqs.size)
            .memory_type_index(memory_type_index);
        let memory = ctx
            .device
            .allocate_memory(&ai, None)
            .expect_log("failed to allocate memory for a buffer");

        ctx.device
            .bind_buffer_memory(buffer, memory, 0)
            .expect_log("failed to bind a buffer to the memory");

        (buffer, memory)
    }
}
