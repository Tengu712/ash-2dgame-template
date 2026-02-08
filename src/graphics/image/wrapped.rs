use super::super::Context;
use ash::vk;

/// 既存のイメージをラッピングしたイメージ
///
/// 実際はただのイメージビューのコンテナ。
/// `OwnedImage`との対比で定義しているだけで、
/// `OwnedImage`と同列に扱うこともない。
pub struct WrappedImage {
    pub view: vk::ImageView,
}

impl WrappedImage {
    pub fn from(
        ctx: &Context,
        image: vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
        components: vk::ComponentMapping,
    ) -> Self {
        Self {
            view: super::create_image_view(&ctx.device, image, format, aspect, components),
        }
    }

    pub fn destroy(self, ctx: &Context) {
        unsafe {
            ctx.device.destroy_image_view(self.view, None);
        }
    }
}
