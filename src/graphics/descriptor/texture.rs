use crate::logs::LoggableResult;

use super::{super::context::Context, update, utils};
use ash::{Device, vk};
use std::{marker::PhantomData, ptr};

/// このディスクリプタセットにおけるバインディング
///
/// NOTE: 必ずシェーダ側と同期すること。
pub(super) const BINDINGS: &[vk::DescriptorSetLayoutBinding] = &[
    // tex
    vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::SAMPLED_IMAGE,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
        p_immutable_samplers: ptr::null(),
        _marker: PhantomData,
    },
    // smplr
    vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: vk::DescriptorType::SAMPLER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
        p_immutable_samplers: ptr::null(),
        _marker: PhantomData,
    },
];

/// テクスチャマッピングに関するディスクリプタセットおよびそのリソース
///
/// NOTE: イメージリソースは外部で持つ。
pub struct TextureMapping {
    pub layout: vk::DescriptorSetLayout,
    pub set: vk::DescriptorSet,
    pub sampler: vk::Sampler,
}

impl TextureMapping {
    pub fn new(ctx: &Context, pool: vk::DescriptorPool) -> Self {
        let layout = utils::create_descriptor_set_layout(&ctx.device, BINDINGS);
        let set = utils::allocate_descriptor_set(&ctx.device, pool, layout);
        let sampler = create_sampler(&ctx.device, true, true, false);

        update::update_descriptor_sets(
            &ctx.device,
            &[update::Info::from_sampler(&[sampler], set, &BINDINGS[1])],
        );

        Self {
            layout,
            set,
            sampler,
        }
    }

    pub fn destroy(self, ctx: &Context) {
        unsafe {
            ctx.device.destroy_sampler(self.sampler, None);
            ctx.device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

impl TextureMapping {
    pub fn update(&self, ctx: &Context, image_view: vk::ImageView) {
        update::update_descriptor_sets(
            &ctx.device,
            &[update::Info::from_image_view(
                &[image_view],
                self.set,
                &BINDINGS[0],
            )],
        );
    }
}

fn create_sampler(
    device: &Device,
    linear_mag: bool,
    linear_min: bool,
    repeat: bool,
) -> vk::Sampler {
    let ci = vk::SamplerCreateInfo::default()
        .mag_filter(if linear_mag {
            vk::Filter::LINEAR
        } else {
            vk::Filter::NEAREST
        })
        .min_filter(if linear_min {
            vk::Filter::LINEAR
        } else {
            vk::Filter::NEAREST
        })
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .address_mode_u(if repeat {
            vk::SamplerAddressMode::REPEAT
        } else {
            vk::SamplerAddressMode::CLAMP_TO_EDGE
        })
        .address_mode_v(if repeat {
            vk::SamplerAddressMode::REPEAT
        } else {
            vk::SamplerAddressMode::CLAMP_TO_EDGE
        })
        .max_lod(vk::LOD_CLAMP_NONE);
    unsafe {
        device
            .create_sampler(&ci, None)
            .expect_log("failed to create a sampler")
    }
}
