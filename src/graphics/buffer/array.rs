use super::super::context::Context;
use ash::{prelude::VkResult, vk};
use std::{marker::PhantomData, mem, ptr, rc::Rc};

pub struct ArrayBuffer<T> {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    ctx: Rc<Context>,
    _data: PhantomData<T>,
}

impl<T> ArrayBuffer<T> {
    pub fn new(ctx: Rc<Context>, count: usize, usage: vk::BufferUsageFlags) -> VkResult<Self> {
        let size = count * size_of::<T>();
        let size = size as vk::DeviceSize;
        let buffer = super::create_buffer(&ctx.device, size, usage)?;
        let memory = super::allocate_memory(&ctx, buffer)?;
        unsafe { ctx.device.bind_buffer_memory(buffer, memory, 0)? };
        Ok(Self {
            buffer,
            memory,
            ctx,
            _data: PhantomData,
        })
    }

    pub fn copy_to_memory(&self, src: &[T], offset: usize) -> VkResult<()> {
        unsafe {
            let offset = offset * size_of::<T>();
            let offset = offset as vk::DeviceSize;
            let size = mem::size_of_val(src);
            let size = size as vk::DeviceSize;
            let p = self.ctx.device.map_memory(
                self.memory,
                offset,
                size,
                vk::MemoryMapFlags::empty(),
            )?;
            ptr::copy_nonoverlapping(src.as_ptr(), p as *mut T, src.len());
            self.ctx.device.unmap_memory(self.memory);
            Ok(())
        }
    }
}

impl<T> Drop for ArrayBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.free_memory(self.memory, None);
            self.ctx.device.destroy_buffer(self.buffer, None);
        }
    }
}
