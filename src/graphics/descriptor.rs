use super::context::Context;
use crate::logs::*;
use ash::{Device, vk};

pub mod texture;
pub mod transform;
mod update;
mod utils;

use texture::TextureMapping;
use transform::Transformation;

/// ディスクリプタセットの最大個数
///
/// NOTE: ゲームに応じて適切に設定すべし。
///       基本的にはディスクリプタセットの種類数と同じで良いが、
///       ディスクリプタセットを切り替えようとしたら変わる。
const MAX_DESC_SET_COUNT: u32 = 2;
const ALL_BINDINGS: &[&[vk::DescriptorSetLayoutBinding]] =
    &[transform::BINDINGS, texture::BINDINGS];

pub struct Descriptors {
    pub pool: vk::DescriptorPool,
    pub trans: Transformation,
    pub tex: TextureMapping,
}

impl Descriptors {
    pub fn new(ctx: &Context) -> Self {
        let pool = create_descriptor_pool(&ctx.device);
        let trans = Transformation::new(ctx, pool);
        let tex = TextureMapping::new(ctx, pool);
        Self { pool, trans, tex }
    }

    pub fn destroy(self, ctx: &Context) {
        unsafe {
            self.tex.destroy(ctx);
            self.trans.destroy(ctx);
            ctx.device.destroy_descriptor_pool(self.pool, None);
        }
    }
}

impl Descriptors {
    pub fn collect_set_layouts(&self) -> Vec<vk::DescriptorSetLayout> {
        vec![self.trans.layout, self.tex.layout]
    }

    pub fn record_bind_command(
        &self,
        ctx: &Context,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
    ) {
        unsafe {
            ctx.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &[self.trans.set, self.tex.set],
                &[],
            );
        }
    }
}

fn create_descriptor_pool(device: &Device) -> vk::DescriptorPool {
    const POOL_SIZES: &[vk::DescriptorPoolSize] = &collect_pool_sizes();

    let ci = vk::DescriptorPoolCreateInfo::default()
        .max_sets(MAX_DESC_SET_COUNT)
        .pool_sizes(POOL_SIZES);
    unsafe {
        device
            .create_descriptor_pool(&ci, None)
            .expect_log("failed to create a descriptor pool")
    }
}

const fn collect_pool_sizes() -> [vk::DescriptorPoolSize; 4] {
    const fn count(ty: vk::DescriptorType) -> u32 {
        let mut sum = 0;
        let mut i = 0;
        while i < ALL_BINDINGS.len() {
            let mut j = 0;
            while j < ALL_BINDINGS[i].len() {
                if ALL_BINDINGS[i][j].descriptor_type.as_raw() == ty.as_raw() {
                    sum += ALL_BINDINGS[i][j].descriptor_count;
                }
                j += 1;
            }
            i += 1;
        }
        sum
    }

    macro_rules! pool_size_of {
        ($ty:expr) => {
            vk::DescriptorPoolSize {
                ty: $ty,
                descriptor_count: count($ty),
            }
        };
    }

    [
        pool_size_of!(vk::DescriptorType::UNIFORM_BUFFER),
        pool_size_of!(vk::DescriptorType::STORAGE_BUFFER),
        pool_size_of!(vk::DescriptorType::SAMPLED_IMAGE),
        pool_size_of!(vk::DescriptorType::SAMPLER),
    ]
}
