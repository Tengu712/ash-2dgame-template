use crate::{config::*, window::Window};
use ash::vk;

pub mod buffer;
pub mod context;
pub mod descriptor;
pub mod image;
pub mod renderpass;
pub mod submit;
pub mod swapchain;
pub mod sync;

use context::Context;
use descriptor::{
    Descriptors,
    transform::{Camera, Instance},
};
use renderpass::{RenderAreas, RenderPass};
use submit::{SubmittedSubmitter, Submitter};
use swapchain::Swapchain;
use sync::Synchronizer;

enum SubmitterState {
    Idle(Submitter),
    Submitted(SubmittedSubmitter),
}

impl SubmitterState {
    fn wait(self, ctx: &Context) -> Submitter {
        match self {
            Self::Idle(submitter) => submitter,
            Self::Submitted(waiter) => waiter.wait(ctx),
        }
    }
}

pub struct GraphicsEngine {
    ctx: Context,
    swapchain: Swapchain,
    descriptors: Descriptors,
    render_pass: RenderPass,
    synchronizer: Synchronizer,
    submitter: SubmitterState,
}

impl GraphicsEngine {
    pub fn new(window: &Window) -> Self {
        let ctx = Context::new();
        let swapchain = Swapchain::new(window, &ctx);
        let descriptors = Descriptors::new(&ctx);
        let render_pass = RenderPass::new(&ctx, &swapchain, &descriptors.collect_set_layouts());
        let synchronizer = Synchronizer::new(&ctx, swapchain.images.len());
        let submitter = SubmitterState::Idle(Submitter::new(&ctx));
        Self {
            ctx,
            swapchain,
            descriptors,
            render_pass,
            synchronizer,
            submitter,
        }
    }

    pub fn destroy(self) {
        self.ctx.wait_idle();
        self.submitter.wait(&self.ctx).destroy(&self.ctx);
        self.synchronizer.destroy(&self.ctx);
        self.render_pass.destroy(&self.ctx);
        self.descriptors.destroy(&self.ctx);
        self.swapchain.destroy(&self.ctx, false);
        self.ctx.destroy();
    }
}

impl GraphicsEngine {
    pub fn ensure_idle(self) -> Self {
        self.ctx.wait_idle();
        Self {
            submitter: SubmitterState::Idle(self.submitter.wait(&self.ctx)),
            ..self
        }
    }

    pub fn draw_frame(
        mut self,
        window: &Window,
        instances: Vec<Instance>,
        camera: Option<Camera>,
    ) -> Self {
        // 準備
        let semaphores = self.synchronizer.current();
        let Ok(index) = self
            .swapchain
            .acquire_next_image_index(&self.ctx, semaphores.started_semaphore)
        else {
            return self.ensure_idle().recreate_swapchain(window);
        };
        let render_area = self.swapchain.get_full_rect();
        let areas = RenderAreas {
            render_area,
            viewport: self.swapchain.calc_aspect_corrected_viewport(ASPECT_RATIO),
            scissor: render_area,
        };

        // 前回提出したコマンドがあれば待機
        let submitter = self.submitter.wait(&self.ctx);

        // ディスクリプタ更新
        self.descriptors.upload(&self.ctx, &instances, &camera);

        // 記録
        let recorder = submitter.start(&self.ctx);
        self.descriptors.record_bind_command(
            &self.ctx,
            recorder.command_buffer(),
            self.render_pass.pipeline.layout,
        );
        self.render_pass.record_render_commands(
            &self.ctx,
            recorder.command_buffer(),
            areas,
            instances.len(),
            index,
        );

        // 提出
        let wait_infos = [(
            semaphores.started_semaphore,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        )];
        let signal_semaphores = [semaphores.finished_semaphore];
        let waiter = recorder.submit(&self.ctx, &wait_infos, &signal_semaphores);
        self.submitter = SubmitterState::Submitted(waiter);

        // プレゼンテーション
        let Ok(()) = self.swapchain.queue_presentation_command(
            &self.ctx,
            index,
            semaphores.finished_semaphore,
        ) else {
            return self.ensure_idle().recreate_swapchain(window);
        };

        // 終了
        self.synchronizer = self.synchronizer.next();
        self
    }

    pub fn recreate_swapchain(self, window: &Window) -> Self {
        let swapchain = self.swapchain.recreate(window, &self.ctx);
        let render_pass = self
            .render_pass
            .recreate_framebuffers(&self.ctx, &swapchain);
        let synchronizer = self.synchronizer.recreate_current(&self.ctx);
        Self {
            swapchain,
            render_pass,
            synchronizer,
            ..self
        }
    }
}
