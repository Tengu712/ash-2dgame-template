use super::{buffer::ArrayBuffer, context::Context};
use crate::logs::*;
use ash::{Device, vk};

pub enum Image {
    Owned {
        image: vk::Image,
        memory: vk::DeviceMemory,
        view: vk::ImageView,
    },
    Wrapped {
        image: vk::Image,
        view: vk::ImageView,
    },
}

impl Image {
    pub fn new(
        ctx: &Context,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        aspect: vk::ImageAspectFlags,
    ) -> Self {
        let extent = vk::Extent3D {
            width,
            height,
            depth: 1,
        };
        let (image, memory) = create_image_util(ctx, extent, format, usage);
        let view = create_image_view(&ctx.device, image, format, aspect);
        Self::Owned {
            image,
            memory,
            view,
        }
    }

    pub fn wrap(
        ctx: &Context,
        image: vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
    ) -> Self {
        Self::Wrapped {
            image,
            view: create_image_view(&ctx.device, image, format, aspect),
        }
    }

    pub fn destroy(self, ctx: &Context) {
        match self {
            Self::Owned {
                image,
                memory,
                view,
            } => unsafe {
                ctx.device.destroy_image_view(view, None);
                ctx.device.free_memory(memory, None);
                ctx.device.destroy_image(image, None);
            },
            Self::Wrapped { view, .. } => unsafe {
                ctx.device.destroy_image_view(view, None);
            },
        }
    }
}

impl Image {
    pub fn view(&self) -> vk::ImageView {
        match self {
            Self::Owned { view, .. } => *view,
            Self::Wrapped { view, .. } => *view,
        }
    }
}

impl Image {
    /// バッファ上のデータをイメージにアップロードするメソッド
    pub fn upload(
        &self,
        ctx: &Context,
        command_buffer: vk::CommandBuffer,
        staging: &ArrayBuffer<u8>,
        width: u32,
        height: u32,
    ) {
        match self {
            Self::Owned { image, .. } => {
                upload_image_data(ctx, command_buffer, staging, width, height, *image);
            }
            _ => panic!("Internal error: try to upload data to wrapped image"),
        }
    }
}

fn create_image_util(
    ctx: &Context,
    extent: vk::Extent3D,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
) -> (vk::Image, vk::DeviceMemory) {
    unsafe {
        let ci = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let image = ctx
            .device
            .create_image(&ci, None)
            .expect_log("failed to create an image");

        let reqs = ctx.device.get_image_memory_requirements(image);
        let memory_type_index = ctx
            .find_memory_type_index(reqs.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL)
            .expect_log("failed to find a memory type for creating an image");
        let ai = vk::MemoryAllocateInfo::default()
            .allocation_size(reqs.size)
            .memory_type_index(memory_type_index);
        let memory = ctx
            .device
            .allocate_memory(&ai, None)
            .expect_log("failed to allocate memory for an image");

        ctx.device
            .bind_image_memory(image, memory, 0)
            .expect_log("failed to bind an image to the memory");

        (image, memory)
    }
}

fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    aspect: vk::ImageAspectFlags,
) -> vk::ImageView {
    let ci = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .components(vk::ComponentMapping {
            r: vk::ComponentSwizzle::R,
            g: vk::ComponentSwizzle::G,
            b: vk::ComponentSwizzle::B,
            a: vk::ComponentSwizzle::A,
        })
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: aspect,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    unsafe {
        device
            .create_image_view(&ci, None)
            .expect_log("failed to create an image view")
    }
}

fn upload_image_data(
    ctx: &Context,
    command_buffer: vk::CommandBuffer,
    staging: &ArrayBuffer<u8>,
    width: u32,
    height: u32,
    image: vk::Image,
) {
    unsafe {
        // イメージレイアウトをTRANSFER_DST_OPTIMALに変更
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
        ctx.device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );

        // アップロード
        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            });
        ctx.device.cmd_copy_buffer_to_image(
            command_buffer,
            staging.buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );

        // イメージレイアウトをSHADER_READ_ONLY_OPTIMALに変更
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ);
        ctx.device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }
}
