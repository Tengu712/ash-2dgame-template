use super::context::Context;
use crate::logs::*;
use ash::{Device, vk};

#[derive(Clone, Copy)]
pub struct Semaphores {
    /// スワップチェーンイメージが利用可能になるまで待機するためのセマフォ
    ///
    /// NOTE: スワップチェーンイメージのインデックス取得時にシグナルセマフォに指定すること。
    pub started_semaphore: vk::Semaphore,
    /// 描画が完了するまで待機するためのセマフォ
    ///
    /// NOTE: コマンドバッファ提出時にシグナルセマフォに指定すること。
    pub finished_semaphore: vk::Semaphore,
}

impl Semaphores {
    fn new(ctx: &Context) -> Self {
        Self {
            started_semaphore: create_semaphore(&ctx.device),
            finished_semaphore: create_semaphore(&ctx.device),
        }
    }

    fn destroy(self, ctx: &Context) {
        unsafe {
            ctx.device.destroy_semaphore(self.finished_semaphore, None);
            ctx.device.destroy_semaphore(self.started_semaphore, None);
        }
    }
}

/// 描画用のセマフォ群
///
/// スワップチェーンイメージへの操作の同期を取るためのもの。
pub struct Synchronizer {
    /// 順繰りに取得するためのインデックスカウンタ
    ///
    /// NOTE: 実際に利用可能になるスワップチェーンイメージのインデックスとは無関係。
    counter: usize,
    /// スワップチェーンイメージの数だけあるセマフォ列
    semaphores: Vec<Semaphores>,
}

impl Synchronizer {
    pub fn new(ctx: &Context, count: usize) -> Self {
        Self {
            counter: 0,
            semaphores: (0..count).map(|_| Semaphores::new(ctx)).collect(),
        }
    }

    pub fn destroy(self, ctx: &Context) {
        for semaphores in self.semaphores {
            semaphores.destroy(ctx);
        }
    }
}

impl Synchronizer {
    pub fn recreate_current(self, ctx: &Context) -> Self {
        let mut semaphores = self.semaphores;
        semaphores[self.counter].destroy(ctx);
        semaphores[self.counter] = Semaphores::new(ctx);
        Self { semaphores, ..self }
    }
}

impl Synchronizer {
    pub fn current(&self) -> Semaphores {
        self.semaphores[self.counter]
    }

    pub fn next(self) -> Self {
        Self {
            counter: (self.counter + 1) % self.semaphores.len(),
            semaphores: self.semaphores,
        }
    }
}

fn create_semaphore(device: &Device) -> vk::Semaphore {
    unsafe {
        device
            .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
            .expect_log("failed to create a semaphore")
    }
}
