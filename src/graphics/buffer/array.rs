use super::super::context::Context;
use crate::logs::*;
use ash::vk;
use std::{marker::PhantomData, mem, ptr};

pub struct ArrayBuffer<T> {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    _data: PhantomData<T>,
}

impl<T> ArrayBuffer<T> {
    pub fn new(ctx: &Context, count: usize, usage: vk::BufferUsageFlags) -> Self {
        let size = count * size_of::<T>();
        let size = size as vk::DeviceSize;
        let (buffer, memory) = super::create_buffer_util(ctx, size, usage);
        Self {
            buffer,
            memory,
            _data: PhantomData,
        }
    }

    pub fn destroy(self, ctx: &Context) {
        unsafe {
            ctx.device.free_memory(self.memory, None);
            ctx.device.destroy_buffer(self.buffer, None);
        }
    }
}

impl<T> ArrayBuffer<T> {
    pub fn copy_to_memory(&self, ctx: &Context, src: &[T], offset: usize) {
        unsafe {
            let offset = offset * size_of::<T>();
            let offset = offset as vk::DeviceSize;
            let size = mem::size_of_val(src);
            let size = size as vk::DeviceSize;
            let p = ctx
                .device
                .map_memory(self.memory, offset, size, vk::MemoryMapFlags::empty())
                .expect_log("failed to map a buffer");
            ptr::copy_nonoverlapping(src.as_ptr(), p as *mut T, src.len());
            ctx.device.unmap_memory(self.memory);
        }
    }
}
