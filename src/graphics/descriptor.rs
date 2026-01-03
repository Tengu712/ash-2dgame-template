use super::context::Context;
use crate::logs::*;
use ash::{Device, vk};
use std::{rc::Rc, slice};

pub mod transform;
mod writer;

use transform::Transformation;

pub struct Descriptors {
    pub pool: vk::DescriptorPool,
    pub trans: Transformation,
    ctx: Rc<Context>,
}

impl Descriptors {
    pub fn new(ctx: Rc<Context>, max_insts_count: usize) -> Self {
        let pool = create_descriptor_pool(&ctx.device);
        let trans = Transformation::new(&ctx, pool, max_insts_count);
        Self { pool, trans, ctx }
    }

    pub fn collect_set_layouts(&self) -> Vec<vk::DescriptorSetLayout> {
        vec![self.trans.layout]
    }

    pub fn record_bind_command(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
    ) {
        unsafe {
            let sets = [self.trans.set];
            self.ctx.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &sets,
                &[],
            );
        }
    }
}

impl Drop for Descriptors {
    fn drop(&mut self) {
        unsafe {
            self.ctx
                .device
                .destroy_descriptor_set_layout(self.trans.layout, None);
            self.ctx.device.destroy_descriptor_pool(self.pool, None);
        }
    }
}

fn create_descriptor_pool(device: &Device) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorType::UNIFORM_BUFFER,
        vk::DescriptorType::STORAGE_BUFFER,
    ]
    .iter()
    .map(|&ty| vk::DescriptorPoolSize {
        ty,
        descriptor_count: count_descriptor_type(transform::BINDINGS, ty),
    })
    .collect::<Vec<_>>();
    let ci = vk::DescriptorPoolCreateInfo::default()
        .max_sets(1)
        .pool_sizes(&pool_sizes);
    unsafe {
        device
            .create_descriptor_pool(&ci, None)
            .expect_log("failed to create a descriptor pool")
    }
}

fn count_descriptor_type(
    bindings: &[vk::DescriptorSetLayoutBinding],
    descriptor_type: vk::DescriptorType,
) -> u32 {
    bindings
        .iter()
        .filter(|b| b.descriptor_type == descriptor_type)
        .map(|b| b.descriptor_count)
        .sum()
}

fn create_descriptor_set_layout(
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

fn allocate_descriptor_set(
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
