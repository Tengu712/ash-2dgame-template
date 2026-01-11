//! ビルドスクリプト
//!
//! - 依存パッケージのインストール
//! - Vulkan loaderとのリンク
//! - Vulkan Validation Layersの実行時有効化のためのconfig.toml生成
//! - シェーダファイルのコンパイル
//! - ウィンドウライブラリのビルドおよびリンク

use std::{
    env,
    path::Path,
    process::{Command, Stdio},
};

mod deps;
#[cfg(target_os = "macos")]
mod macos;
mod shader;
mod vulkan;
#[cfg(target_os = "windows")]
mod windows;

fn run(command: &str, args: &[&str]) {
    run_on(&env::current_dir().unwrap(), command, args)
}

fn run_on(dir: &Path, command: &str, args: &[&str]) {
    let status = Command::new(command)
        .args(args)
        .env("CARGO_PROFILE", env::var("PROFILE").unwrap())
        .current_dir(dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();
    assert!(status.success());
}

fn main() {
    deps::install_dependencies();

    shader::compile_shaders();

    vulkan::add_vulkan_lib_path_to_libpath();
    #[cfg(all(debug_assertions, feature = "vvl"))]
    {
        vulkan::copy_vvl_files_to_deps_dir();
        vulkan::generate_config_toml_to_use_vvl();
    }

    #[cfg(target_os = "windows")]
    {
        windows::build_window_library();
        windows::link_window_library();
    }
    #[cfg(target_os = "macos")]
    {
        macos::build_window_library();
        macos::link_window_library();
        macos::copy_vulkan_dylib();
        macos::set_rpath();
    }

    println!("cargo:rerun-if-changed=build.rs");
}
