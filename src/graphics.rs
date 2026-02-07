use crate::{config::*, logs::*, res::Resource, window::Window};
use ash::vk;
use std::collections::HashMap;

pub mod buffer;
pub mod context;
pub mod descriptor;
pub mod image;
pub mod renderpass;
pub mod submit;
pub mod swapchain;
pub mod sync;
mod utils;

use buffer::ArrayBuffer;
use context::Context;
use descriptor::{
    Descriptors,
    transform::{Camera, Instance},
};
use image::Image;
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

    submitter_for_image: Submitter,
    images: HashMap<Resource, Image>,
}

impl GraphicsEngine {
    pub fn new(window: &Window) -> Self {
        let ctx = Context::new();
        let swapchain = Swapchain::new(window, &ctx);
        let descriptors = Descriptors::new(&ctx);
        let render_pass = RenderPass::new(&ctx, &swapchain, &descriptors.collect_set_layouts());
        let synchronizer = Synchronizer::new(&ctx, swapchain.images.len());
        let submitter = SubmitterState::Idle(Submitter::new(&ctx));
        let submitter_for_image = Submitter::new(&ctx);
        Self {
            ctx,
            swapchain,
            descriptors,
            render_pass,
            synchronizer,
            submitter,
            submitter_for_image,
            images: HashMap::new(),
        }
    }

    pub fn destroy(self) {
        self.ctx.wait_idle();
        for (_, image) in self.images {
            image.destroy(&self.ctx);
        }
        self.submitter_for_image.destroy(&self.ctx);
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

    /// 描画関数
    ///
    /// * window - ウィンドウ。ウィンドウサイズ変化時のスワップチェーンイメージ再作成に必要。
    /// * instances - 描画するインスタンスデータ列。
    /// * camera - カメラデータ。Noneであればアップロードされない。
    /// * images - 更新するイメージディスクリプタ情報列。組(リソースID, ディスクリプタオフセット)。未ロードイメージはスキップされる。
    pub fn draw_frame(
        mut self,
        window: &Window,
        instances: &[Instance],
        camera: &Option<Camera>,
        images: &[(Resource, u32)],
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
        self.descriptors.trans.upload(&self.ctx, instances, camera);
        for (image, offset) in images {
            if let Some(image) = self.images.get(image) {
                self.descriptors
                    .tex
                    .update(&self.ctx, image.view(), *offset);
            }
        }

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

impl GraphicsEngine {
    /// 画像をロードするメソッド
    ///
    /// 失敗時は警告レベルでログを取り、何もしない。
    pub fn load_image(mut self, res: &Resource) -> Self {
        if self.images.contains_key(res) {
            return self;
        }

        // PNGデコード
        let (data, width, height) = match utils::decode_png(res.0) {
            Ok(v) => v,
            Err(e) => {
                warn(&e.to_string());
                return self;
            }
        };

        // イメージ作成
        let image = Image::new(
            &self.ctx,
            width,
            height,
            vk::Format::R8G8B8A8_SRGB, // NOTE: ファイルによってはこのフォーマットじゃないかもね
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            vk::ImageAspectFlags::COLOR,
        );

        // ステージングバッファ作成&アップロード
        let staging =
            ArrayBuffer::<u8>::new(&self.ctx, data.len(), vk::BufferUsageFlags::TRANSFER_SRC);
        staging.copy_to_memory(&self.ctx, &data, 0);

        // アップロード
        let recording = self.submitter_for_image.start(&self.ctx);
        image.upload(
            &self.ctx,
            recording.command_buffer(),
            &staging,
            width,
            height,
        );
        let waiter = recording.submit(&self.ctx, &[], &[]);
        self.submitter_for_image = waiter.wait(&self.ctx);

        // 終了
        staging.destroy(&self.ctx);
        self.images.insert(*res, image);
        self
    }
}
