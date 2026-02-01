use super::context::Context;
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
