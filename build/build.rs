//! ビルドスクリプト
//!
//! 概ねビルド時に実行される。
//! 次を行う:
//! - Conan2による次の依存パッケージのインストール
//!   - Vulkan-Loader
//!   - Vulkan-ValidationLayers (デバッグビルド時)
//!   - glslang
//! - Vulkan loaderとのリンク
//! - Vulkan Validation Layers追加のための実行時環境変数追加
//! - シェーダファイルのコンパイル
//! - ウィンドウライブラリのビルドおよびリンク

use std::{
    env,
    process::{Command, Stdio},
};

mod conan;
#[cfg(target_os = "macos")]
mod macos;
mod shader;
#[cfg(target_os = "windows")]
mod windows;

fn is_command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run(command: &str, args: &[&str]) {
    let status = Command::new(command)
        .args(args)
        .env("CARGO_PROFILE", env::var("PROFILE").unwrap())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();
    assert!(status.success());
}

fn main() {
    conan::install_dependencies();
    conan::generate_cargo_config();
    conan::add_vulkan_lib_dirpath_to_libpath();

    shader::compile_shaders(&conan::get_glslang_bin_dirpath());

    #[cfg(target_os = "windows")]
    {
        windows::build_window_library();
        windows::link_window_library();
    }
    #[cfg(target_os = "macos")]
    {
        // TODO: build_window_library()
        macos::copy_vulkan_dylib(config.get("VULKAN_LIB").unwrap());
        macos::print_link_info();
    }

    println!("cargo:rerun-if-changed=build.rs");
}
