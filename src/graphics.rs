use crate::{
    config::*,
    logs::*,
    res::{CHAR_ATLAS, Resource},
    window::Window,
};
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
use image::{owned::OwnedImage, *};
use renderpass::{RenderAreas, RenderPass};
use submit::{SubmittedSubmitter, Submitter};
use swapchain::Swapchain;
use sync::Synchronizer;

/// 文字テクスチャアトラスの一辺の長さ [px]
pub const CHAR_ATLAS_SIZE: u32 = 512;
/// 文字テクスチャアトラスのチャンネル数
//
// NOTE: macOSではswizzleをゼロコストで使えないので、
//       文字テクスチャアトラスも4チャンネルにする。
pub const CHAR_ATLAS_CHANNEL_COUNT: usize = if cfg!(target_os = "macos") { 4 } else { 1 };
/// 文字テクスチャアトラスのフォーマット
//
// NOTE: 上記の理由でmacOSでは普通の画像と同じフォーマット。
pub const CHAR_ATLAS_FORMAT: vk::Format = if cfg!(target_os = "macos") {
    vk::Format::R8G8B8A8_SRGB
} else {
    vk::Format::R8_UNORM
};
/// 文字テクスチャアトラスのチャンネル数
//
// NOTE: 上記の理由でmacOSでは普通のコンポーネントマッピング。
pub const CHAR_ATLAS_COMPONENT_MAP: vk::ComponentMapping = if cfg!(target_os = "macos") {
    RGBA_COMPONENT_MAP
} else {
    R_COMPONENT_MAP
};

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
    images: HashMap<Resource, OwnedImage>,
}

impl GraphicsEngine {
    pub fn new(window: &Window) -> Self {
        let ctx = Context::new();
        let swapchain = Swapchain::new(window, &ctx);
        let descriptors = Descriptors::new(&ctx);
        let render_pass = RenderPass::new(&ctx, &swapchain, &descriptors.collect_set_layouts());
        let synchronizer = Synchronizer::new(&ctx, swapchain.images.len());
        let submitter = SubmitterState::Idle(Submitter::new(&ctx));

        let mut submitter_for_image = Submitter::new(&ctx);
        let mut images = HashMap::new();
        let char_atlas = OwnedImage::new(
            &ctx,
            CHAR_ATLAS_SIZE,
            CHAR_ATLAS_SIZE,
            CHAR_ATLAS_FORMAT,
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            vk::ImageAspectFlags::COLOR,
            CHAR_ATLAS_COMPONENT_MAP,
        );
        submitter_for_image = init_image(&ctx, submitter_for_image, &char_atlas);
        images.insert(CHAR_ATLAS, char_atlas);

        Self {
            ctx,
            swapchain,
            descriptors,
            render_pass,
            synchronizer,
            submitter,
            submitter_for_image,
            images,
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
    /// * instances - 描画するインスタンスデータ列。超過分は無視される。
    /// * camera - カメラデータ。Noneであればアップロードされない。
    /// * images - 更新するイメージディスクリプタ情報列。組(リソースID, ディスクリプタオフセット)。未ロードイメージは無視される。超過分は無視される。
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
                self.descriptors.tex.update(&self.ctx, image.view, *offset);
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
        let image = OwnedImage::new(
            &self.ctx,
            width,
            height,
            vk::Format::R8G8B8A8_SRGB, // NOTE: ファイルによってはこのフォーマットじゃないかもね
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            vk::ImageAspectFlags::COLOR,
            RGBA_COMPONENT_MAP,
        );
        self.submitter_for_image = init_image(&self.ctx, self.submitter_for_image, &image);

        // アップロード
        self.submitter_for_image = upload_image(
            &self.ctx,
            self.submitter_for_image,
            &image,
            &data,
            vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D { width, height },
            },
        );

        // 終了
        self.images.insert(*res, image);
        self
    }

    /// 文字テクスチャアトラスにデータをアップロードするメソッド
    pub fn upload_char(mut self, data: &[u8], x: i32, y: i32, width: u32, height: u32) -> Self {
        let image = self
            .images
            .get(&CHAR_ATLAS)
            .expect("Internal error: char atlas not found");
        self.submitter_for_image = upload_image(
            &self.ctx,
            self.submitter_for_image,
            image,
            data,
            vk::Rect2D {
                offset: vk::Offset2D { x, y },
                extent: vk::Extent2D { width, height },
            },
        );
        self
    }
}

fn init_image(ctx: &Context, submitter: Submitter, image: &OwnedImage) -> Submitter {
    let recorder = submitter.start(ctx);
    image.record_pipeline_barrier_command(
        ctx,
        recorder.command_buffer(),
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    let waiter = recorder.submit(ctx, &[], &[]);
    waiter.wait(ctx)
}

fn upload_image(
    ctx: &Context,
    submitter: Submitter,
    image: &OwnedImage,
    data: &[u8],
    area: vk::Rect2D,
) -> Submitter {
    let staging = ArrayBuffer::<u8>::new(ctx, data.len(), vk::BufferUsageFlags::TRANSFER_SRC);
    staging.copy_to_memory(ctx, data, 0);

    let recording = submitter.start(ctx);
    image.record_upload_command(ctx, recording.command_buffer(), staging.buffer, area);
    let waiter = recording.submit(ctx, &[], &[]);
    let submitter = waiter.wait(ctx);

    staging.destroy(ctx);
    submitter
}
