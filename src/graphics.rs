use crate::settings::*;
use ash::Entry;
use std::sync::Arc;

mod context;
mod submit;
mod window;

use context::Context;
use submit::Submitter;
use window::Window;

/// アプリケーションのグラフィックスを司るオブジェクト
///
/// WARN: 1アプリケーション上で1インスタンスのみ作成すること。
//
// NOTE: dropの順番が非常に重要なので、適切な順番にメンバを宣言している。
pub struct GraphicsEngine {
    submitter: Submitter,
    ctx: Arc<Context>,
    entry: Entry,
    window: Window,
}

impl GraphicsEngine {
    pub fn new() -> Self {
        let window = Window::new(WINDOW_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT);
        let entry = Entry::linked();
        let ctx = Context::new(&entry, APPLICATION_NAME, APPLICATION_VERSION);
        let ctx = Arc::new(ctx);
        let submitter = Submitter::new(Arc::clone(&ctx));
        Self {
            submitter,
            ctx,
            entry,
            window,
        }
    }

    /// 1フレームを実行する関数
    ///
    /// ウィンドウが閉じられた場合はfalseを、そうでない場合はtrueを返す。
    pub fn run(&self) -> bool {
        if !self.window.process_events() {
            return false;
        }

        // DEBUG:
        std::thread::sleep(std::time::Duration::from_millis(16));
        true
    }
}
