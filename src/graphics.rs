use crate::settings::*;
use ash::Entry;

mod context;

use context::Context;

/// アプリケーションのグラフィックスを司るオブジェクト群
///
/// WARN: 1アプリケーション上で1インスタンスのみ作成すること。
pub struct GraphicsEngine {
    ctx: Context,

    // NOTE: 上記オブジェクトより先にdropするとACCESS VIOLATIONが発生するので、
    //       上記オブジェクトより後にdropするために最後に宣言する。
    entry: Entry,
}

impl GraphicsEngine {
    pub fn new() -> Self {
        let entry = Entry::linked();
        let ctx = Context::new(&entry, APPLICATION_NAME, APPLICATION_VERSION);
        Self { ctx, entry }
    }
}
