use super::{super::framebuffer::Framebuffer, RenderPass, Semaphores};
use ash::{prelude::VkResult, vk};

pub struct RecordingRenderPass<'a>(pub(super) &'a mut RenderPass);

impl<'a> RecordingRenderPass<'a> {
    pub fn swapchain_image_started_semaphore(&self) -> vk::Semaphore {
        self.0.semaphores[self.0.semaphores_counter].started_semaphore
    }

    pub fn record_render_commands(
        self,
        command_buffer: vk::CommandBuffer,
        framebuffer: &Framebuffer,
        area: vk::Rect2D,
    ) -> VkResult<Semaphores> {
        unsafe {
            // レンダーパス開始
            let bi = vk::RenderPassBeginInfo::default()
                .render_pass(self.0.render_pass)
                .framebuffer(framebuffer.framebuffer)
                .render_area(area)
                .clear_values(&framebuffer.clear_colors);
            self.0.ctx.device.cmd_begin_render_pass(
                command_buffer,
                &bi,
                vk::SubpassContents::INLINE,
            );

            // パイプラインバインド
            self.0.ctx.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.0.pipeline.pipeline,
            );

            // ビューポート設定
            let viewports = [vk::Viewport {
                x: area.offset.x as f32,
                y: area.offset.y as f32,
                width: area.extent.width as f32,
                height: area.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];
            let scissors = [area];
            self.0
                .ctx
                .device
                .cmd_set_viewport(command_buffer, 0, &viewports);
            self.0
                .ctx
                .device
                .cmd_set_scissor(command_buffer, 0, &scissors);

            // DEBUG:
            self.0.ctx.device.cmd_draw(command_buffer, 4, 1, 0, 0);

            // レンダーパス終了
            self.0.ctx.device.cmd_end_render_pass(command_buffer);

            Ok(self.0.semaphores[self.0.semaphores_counter])
        }
    }
}

impl Drop for RecordingRenderPass<'_> {
    fn drop(&mut self) {
        self.0.semaphores_counter = (self.0.semaphores_counter + 1) % self.0.semaphores.len();
    }
}
