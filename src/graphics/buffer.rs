use super::context::Context;
use ash::{Device, prelude::VkResult, vk};

pub mod array;
pub mod single;

fn create_buffer(
    device: &Device,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
) -> VkResult<vk::Buffer> {
    let ci = vk::BufferCreateInfo::default().size(size).usage(usage);
    unsafe { device.create_buffer(&ci, None) }
}

fn allocate_memory(ctx: &Context, buffer: vk::Buffer) -> VkResult<vk::DeviceMemory> {
    let reqs = unsafe { ctx.device.get_buffer_memory_requirements(buffer) };
    let memory_type_index = ctx
        .find_memory_type_index(reqs.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE)
        .ok_or(vk::Result::ERROR_OUT_OF_DEVICE_MEMORY)?;
    let ai = vk::MemoryAllocateInfo::default()
        .allocation_size(reqs.size)
        .memory_type_index(memory_type_index);
    unsafe { ctx.device.allocate_memory(&ai, None) }
}
