use ash::{Device, vk};
use std::slice;

struct BufferInfo {
    bi: vk::DescriptorBufferInfo,
    set: vk::DescriptorSet,
    binding: u32,
    count: u32,
    ty: vk::DescriptorType,
}

pub struct DescriptorWriter {
    bis: Vec<BufferInfo>,
}

impl DescriptorWriter {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bis: Vec::with_capacity(capacity),
        }
    }

    pub fn push_buffer(
        &mut self,
        set: vk::DescriptorSet,
        buffer: vk::Buffer,
        binding: &vk::DescriptorSetLayoutBinding,
    ) {
        let bi = BufferInfo {
            bi: vk::DescriptorBufferInfo {
                buffer,
                offset: 0,
                range: vk::WHOLE_SIZE,
            },
            set,
            binding: binding.binding,
            count: binding.descriptor_count,
            ty: binding.descriptor_type,
        };
        self.bis.push(bi);
    }

    pub fn update(self, device: &Device) {
        let mut writes = Vec::with_capacity(self.bis.len());
        for bi in self.bis.iter() {
            let write = vk::WriteDescriptorSet::default()
                .dst_set(bi.set)
                .dst_binding(bi.binding)
                .descriptor_count(bi.count)
                .descriptor_type(bi.ty)
                .buffer_info(slice::from_ref(&bi.bi));
            writes.push(write);
        }
        unsafe {
            device.update_descriptor_sets(&writes, &[]);
        }
    }
}
