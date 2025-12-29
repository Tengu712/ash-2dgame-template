use super::super::context::Context;
use ash::{prelude::VkResult, vk};
use std::rc::Rc;

pub struct ImageView {
    pub image: vk::Image,
    pub view: vk::ImageView,
    ctx: Rc<Context>,
}

impl ImageView {
    pub fn from(
        ctx: Rc<Context>,
        image: vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
    ) -> VkResult<Self> {
        let view = super::create_image_view(&ctx.device, image, format, aspect)?;
        Ok(Self { image, view, ctx })
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_image_view(self.view, None);
        }
    }
}
