use super::{
    super::{
        buffer::{ArrayBuffer, Buffer},
        context::Context,
    },
    writer::DescriptorWriter,
};
use crate::logs::*;
use ash::vk;
use glam::{Mat4, Vec4};
use std::{marker::PhantomData, ptr, rc::Rc};

pub(super) const BINDINGS: &[vk::DescriptorSetLayoutBinding] = &[
    // instances
    vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
        _marker: PhantomData,
    },
    // camera
    vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
        _marker: PhantomData,
    },
];

#[repr(C, align(16))]
#[derive(Debug, Default, Clone, Copy)]
pub struct Instance {
    pub transform: Mat4,
    pub color: Vec4,
}

#[repr(C, align(16))]
#[derive(Debug, Default, Clone, Copy)]
pub struct Camera {
    pub view: Mat4,
    pub proj: Mat4,
}

/// 変形に関するディスクリプタセットおよびそのリソース
pub struct Transformation {
    pub layout: vk::DescriptorSetLayout,
    pub set: vk::DescriptorSet,
    pub insts_buffer: ArrayBuffer<Instance>,
    pub camera_buffer: Buffer<Camera>,
}

impl Transformation {
    pub fn new(ctx: &Rc<Context>, pool: vk::DescriptorPool, max_insts_count: usize) -> Self {
        let layout = super::create_descriptor_set_layout(&ctx.device, BINDINGS);
        let set = super::allocate_descriptor_set(&ctx.device, pool, layout);

        let insts_buffer = ArrayBuffer::new(
            Rc::clone(ctx),
            max_insts_count,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        )
        .expect_log("failed to create a instance buffer");
        let camera_buffer = Buffer::new(Rc::clone(ctx), vk::BufferUsageFlags::UNIFORM_BUFFER)
            .expect_log("failed to create a camera buffer");

        let mut writer = DescriptorWriter::with_capacity(BINDINGS.len());
        writer.push_buffer(set, insts_buffer.buffer, &BINDINGS[0]);
        writer.push_buffer(set, camera_buffer.buffer, &BINDINGS[1]);
        writer.update(&ctx.device);

        Self {
            layout,
            set,
            insts_buffer,
            camera_buffer,
        }
    }
}
