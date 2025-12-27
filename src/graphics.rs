use crate::settings::*;
use ash::Entry;
use std::sync::Arc;

mod context;
mod submit;

use context::Context;
use submit::Submitter;

/// アプリケーションのグラフィックスを司るオブジェクト
///
/// WARN: 1アプリケーション上で1インスタンスのみ作成すること。
pub struct GraphicsEngine {
    submitter: Submitter,

    // NOTE: 上記オブジェクトより後にdropするために最後に宣言する。
    ctx: Arc<Context>,

    // NOTE: Contextより先にdropするとACCESS VIOLATIONが発生するので、
    //       Contextより後にdropするために最後に宣言する。
    entry: Entry,
}

impl GraphicsEngine {
    pub fn new() -> Self {
        let entry = Entry::linked();
        let ctx = Context::new(&entry, APPLICATION_NAME, APPLICATION_VERSION);
        let ctx = Arc::new(ctx);
        let submitter = Submitter::new(Arc::clone(&ctx));
        Self {
            submitter,
            ctx,
            entry,
        }
    }
}
