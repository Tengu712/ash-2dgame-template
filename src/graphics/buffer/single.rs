use super::super::context::Context;
use ash::{prelude::VkResult, vk};
use std::{marker::PhantomData, mem, ptr, rc::Rc};

pub struct Buffer<T> {
    pub size: vk::DeviceSize,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    ctx: Rc<Context>,
    _data: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new(ctx: Rc<Context>, usage: vk::BufferUsageFlags) -> VkResult<Self> {
        let size = size_of::<T>();
        let size = size as vk::DeviceSize;
        let buffer = super::create_buffer(&ctx.device, size, usage)?;
        let memory = super::allocate_memory(&ctx, buffer)?;
        unsafe { ctx.device.bind_buffer_memory(buffer, memory, 0)? };
        Ok(Self {
            size,
            buffer,
            memory,
            ctx,
            _data: PhantomData,
        })
    }

    pub fn copy_to_memory(&self, src: &T) -> VkResult<()> {
        unsafe {
            let size = mem::size_of::<T>();
            let size = size as vk::DeviceSize;
            let p =
                self.ctx
                    .device
                    .map_memory(self.memory, 0, size, vk::MemoryMapFlags::empty())?;
            ptr::copy_nonoverlapping(src as *const T, p as *mut T, 1);
            self.ctx.device.unmap_memory(self.memory);
            Ok(())
        }
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.free_memory(self.memory, None);
            self.ctx.device.destroy_buffer(self.buffer, None);
        }
    }
}
