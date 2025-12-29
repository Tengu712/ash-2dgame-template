use super::super::context::Context;
use ash::{prelude::VkResult, vk};
use std::rc::Rc;

/// 自身でリソースを持つイメージおよびそのビュー
pub struct OwnedImage {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    ctx: Rc<Context>,
}

impl OwnedImage {
    pub fn new(
        ctx: Rc<Context>,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        aspect: vk::ImageAspectFlags,
    ) -> VkResult<Self> {
        unsafe {
            let extent = vk::Extent3D {
                width,
                height,
                depth: 1,
            };

            // image
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
            let image = ctx.device.create_image(&ci, None)?;

            // memory
            let reqs = ctx.device.get_image_memory_requirements(image);
            let memory_type_index = ctx
                .find_memory_type_index(
                    reqs.memory_type_bits,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                )
                .ok_or(vk::Result::ERROR_OUT_OF_DEVICE_MEMORY)?;
            let ai = vk::MemoryAllocateInfo::default()
                .allocation_size(reqs.size)
                .memory_type_index(memory_type_index);
            let memory = ctx.device.allocate_memory(&ai, None)?;

            // view
            let view = super::create_image_view(&ctx.device, image, format, aspect)?;

            // bind image and memory
            ctx.device.bind_image_memory(image, memory, 0)?;

            Ok(Self {
                image,
                memory,
                view,
                ctx,
            })
        }
    }
}

impl Drop for OwnedImage {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_image_view(self.view, None);
            self.ctx.device.free_memory(self.memory, None);
            self.ctx.device.destroy_image(self.image, None);
        }
    }
}
