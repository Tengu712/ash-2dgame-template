use crate::logs::*;
use ash::{Device, vk};
use std::slice;

pub fn create_descriptor_set_layout(
    device: &Device,
    bindings: &[vk::DescriptorSetLayoutBinding],
) -> vk::DescriptorSetLayout {
    let ci = vk::DescriptorSetLayoutCreateInfo::default().bindings(bindings);
    unsafe {
        device
            .create_descriptor_set_layout(&ci, None)
            .expect_log("failed to create a descriptor set layout")
    }
}

pub fn allocate_descriptor_set(
    device: &Device,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
) -> vk::DescriptorSet {
    let ai = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(slice::from_ref(&layout));
    let sets = unsafe {
        device
            .allocate_descriptor_sets(&ai)
            .expect_log("failed to allocate a descriptor set")
    };
    if sets.is_empty() {
        panic_log("failed to allocate a descriptor set");
    } else {
        sets[0]
    }
}
