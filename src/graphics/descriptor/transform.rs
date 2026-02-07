use super::{
    super::{
        buffer::{ArrayBuffer, Buffer},
        context::Context,
    },
    update, utils,
};
use ash::vk;
use glam::{Mat4, Vec4};
use std::{marker::PhantomData, ptr};

/// インスタンスバッファ長
///
/// NOTE: 1ドローコールで描画できる最大のインスタンス数。
///       ゲームに応じて適切に設定すること。
const MAX_INSTANCE_COUNT: usize = 32;

/// このディスクリプタセットにおけるバインディング
///
/// NOTE: 必ずシェーダ側と同期すること。
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
    pub uv: Vec4,
}

#[repr(C, align(16))]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
    pub fn new(ctx: &Context, pool: vk::DescriptorPool) -> Self {
        let layout = utils::create_descriptor_set_layout(&ctx.device, BINDINGS);
        let set = utils::allocate_descriptor_set(&ctx.device, pool, layout);

        let insts_buffer = ArrayBuffer::new(
            ctx,
            MAX_INSTANCE_COUNT,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        );
        let camera_buffer = Buffer::new(ctx, vk::BufferUsageFlags::UNIFORM_BUFFER);

        update::update_descriptor_sets(
            &ctx.device,
            &[
                update::Info::from_buffer(&[insts_buffer.buffer], set, &BINDINGS[0]),
                update::Info::from_buffer(&[camera_buffer.buffer], set, &BINDINGS[1]),
            ],
        );

        Self {
            layout,
            set,
            insts_buffer,
            camera_buffer,
        }
    }

    pub fn destroy(self, ctx: &Context) {
        unsafe {
            ctx.device.destroy_descriptor_set_layout(self.layout, None);
            self.camera_buffer.destroy(ctx);
            self.insts_buffer.destroy(ctx);
        }
    }
}

impl Transformation {
    pub fn upload(&self, ctx: &Context, instances: &[Instance], camera: &Option<Camera>) {
        if !instances.is_empty() {
            self.insts_buffer.copy_to_memory(
                ctx,
                &instances[..instances.len().min(MAX_INSTANCE_COUNT)],
                0,
            );
        }
        if let Some(camera) = camera {
            self.camera_buffer.copy_to_memory(ctx, camera);
        }
    }
}
