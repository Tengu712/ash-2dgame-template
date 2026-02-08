use crate::logs::*;
use ash::{Device, vk};

pub mod owned;
pub mod wrapped;

pub const RGBA_COMPONENT_MAP: vk::ComponentMapping = vk::ComponentMapping {
    r: vk::ComponentSwizzle::R,
    g: vk::ComponentSwizzle::G,
    b: vk::ComponentSwizzle::B,
    a: vk::ComponentSwizzle::A,
};
pub const R_COMPONENT_MAP: vk::ComponentMapping = vk::ComponentMapping {
    r: vk::ComponentSwizzle::ONE,
    g: vk::ComponentSwizzle::ONE,
    b: vk::ComponentSwizzle::ONE,
    a: vk::ComponentSwizzle::R,
};

fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    aspect: vk::ImageAspectFlags,
    components: vk::ComponentMapping,
) -> vk::ImageView {
    let ci = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .components(components)
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
