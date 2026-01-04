use ash::{khr::surface, prelude::VkResult, vk};

const DESIRED_MIN_IMAGE_COUNT: u32 = 2;

const DESIRED_FORMATS: &[vk::SurfaceFormatKHR] = &[
    vk::SurfaceFormatKHR {
        format: vk::Format::B8G8R8A8_SRGB,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    },
    vk::SurfaceFormatKHR {
        format: vk::Format::R8G8B8A8_SRGB,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    },
    vk::SurfaceFormatKHR {
        format: vk::Format::B8G8R8A8_UNORM,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    },
    vk::SurfaceFormatKHR {
        format: vk::Format::R8G8B8A8_UNORM,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    },
];

const DESIRED_PRESENT_MODE: vk::PresentModeKHR = vk::PresentModeKHR::MAILBOX;
const ALTERNATIVE_PRESENT_MODE: vk::PresentModeKHR = vk::PresentModeKHR::FIFO;

const DESIRED_PRE_TRANSFORM: vk::SurfaceTransformFlagsKHR = vk::SurfaceTransformFlagsKHR::IDENTITY;

pub struct SurfaceInfoForSwapchain {
    pub resolution: vk::Extent2D,
    pub min_image_count: u32,
    pub format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub pre_transform: vk::SurfaceTransformFlagsKHR,
}

impl SurfaceInfoForSwapchain {
    pub fn from(
        instance: &surface::Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        window_size: vk::Extent2D,
    ) -> VkResult<Self> {
        unsafe {
            let caps =
                instance.get_physical_device_surface_capabilities(physical_device, surface)?;

            // resolution
            let resolution = if caps.current_extent.width == u32::MAX {
                vk::Extent2D {
                    width: window_size
                        .width
                        .clamp(caps.min_image_extent.width, caps.max_image_extent.width),
                    height: window_size
                        .height
                        .clamp(caps.min_image_extent.height, caps.max_image_extent.height),
                }
            } else {
                caps.current_extent
            };

            // min_image_count
            let min_image_count = if caps.max_image_count > 0 {
                DESIRED_MIN_IMAGE_COUNT
                    .max(caps.min_image_count)
                    .min(caps.max_image_count)
            } else {
                DESIRED_MIN_IMAGE_COUNT.max(caps.min_image_count)
            };

            // format
            let formats = instance.get_physical_device_surface_formats(physical_device, surface)?;
            if formats.is_empty() {
                return Err(vk::Result::ERROR_SURFACE_LOST_KHR);
            }
            let format = DESIRED_FORMATS
                .iter()
                .find(|&desired| {
                    formats.iter().any(|format| {
                        desired.format == format.format && desired.color_space == format.color_space
                    })
                })
                .copied()
                .unwrap_or(formats[0]);

            // present_mode
            let present_modes =
                instance.get_physical_device_surface_present_modes(physical_device, surface)?;
            let present_mode = present_modes
                .iter()
                .find(|&&mode| mode == DESIRED_PRESENT_MODE)
                .copied()
                .unwrap_or(ALTERNATIVE_PRESENT_MODE);

            // pre_transform
            let pre_transform = if caps.supported_transforms.contains(DESIRED_PRE_TRANSFORM) {
                DESIRED_PRE_TRANSFORM
            } else {
                caps.current_transform
            };

            Ok(Self {
                resolution,
                min_image_count,
                format,
                present_mode,
                pre_transform,
            })
        }
    }
}
