use super::super::Context;
use crate::logs::*;
use ash::vk;

/// イメージ
pub struct OwnedImage {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
}

impl OwnedImage {
    pub fn new(
        ctx: &Context,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        aspect: vk::ImageAspectFlags,
        components: vk::ComponentMapping,
    ) -> Self {
        unsafe {
            let extent = vk::Extent3D {
                width,
                height,
                depth: 1,
            };

            // イメージ作成
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

            // アロケート
            let reqs = ctx.device.get_image_memory_requirements(image);
            let memory_type_index = ctx
                .find_memory_type_index(
                    reqs.memory_type_bits,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                )
                .expect_log("failed to find a memory type for creating an image");
            let ai = vk::MemoryAllocateInfo::default()
                .allocation_size(reqs.size)
                .memory_type_index(memory_type_index);
            let memory = ctx
                .device
                .allocate_memory(&ai, None)
                .expect_log("failed to allocate memory for an image");

            // バインド
            ctx.device
                .bind_image_memory(image, memory, 0)
                .expect_log("failed to bind an image to the memory");

            // ビュー作成
            let view = super::create_image_view(&ctx.device, image, format, aspect, components);

            // 終了
            Self {
                image,
                memory,
                view,
            }
        }
    }

    pub fn destroy(self, ctx: &Context) {
        unsafe {
            ctx.device.destroy_image_view(self.view, None);
            ctx.device.free_memory(self.memory, None);
            ctx.device.destroy_image(self.image, None);
        }
    }
}

impl OwnedImage {
    pub fn record_pipeline_barrier_command(
        &self,
        ctx: &Context,
        command_buffer: vk::CommandBuffer,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        unsafe {
            let barrier = vk::ImageMemoryBarrier::default()
                .old_layout(old_layout)
                .new_layout(new_layout)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.image)
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
        }
    }

    pub fn record_upload_command(
        &self,
        ctx: &Context,
        command_buffer: vk::CommandBuffer,
        src: vk::Buffer,
        area: vk::Rect2D,
    ) {
        unsafe {
            // イメージレイアウトをTRANSFER_DST_OPTIMALに変更
            self.record_pipeline_barrier_command(
                ctx,
                command_buffer,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
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
                .image_offset(vk::Offset3D {
                    x: area.offset.x,
                    y: area.offset.y,
                    z: 0,
                })
                .image_extent(vk::Extent3D {
                    width: area.extent.width,
                    height: area.extent.height,
                    depth: 1,
                });
            ctx.device.cmd_copy_buffer_to_image(
                command_buffer,
                src,
                self.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );

            // イメージレイアウトをSHADER_READ_ONLY_OPTIMALに変更
            self.record_pipeline_barrier_command(
                ctx,
                command_buffer,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            );
        }
    }
}
