//! ビルドスクリプト
//!
//! - 依存パッケージのインストール
//! - Vulkan Validation Layersの実行時有効化のためのconfig.toml生成
//! - シェーダファイルのコンパイル
//! - ウィンドウライブラリのビルドおよびリンク

use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

mod deps;
#[cfg(target_os = "macos")]
mod macos;
mod shader;
#[cfg(all(debug_assertions, feature = "vvl"))]
mod vvl;
mod window;

#[allow(dead_code)]
fn cargo_manifest_path() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
}

#[allow(dead_code)]
fn deps_path() -> PathBuf {
    cargo_manifest_path().join("deps")
}

#[allow(dead_code)]
fn build_path() -> PathBuf {
    cargo_manifest_path()
        .join("target")
        .join(env::var("PROFILE").unwrap())
}

fn run(command: &str, args: &[&str]) {
    run_on(&env::current_dir().unwrap(), command, args)
}

fn run_on(dir: &Path, command: &str, args: &[&str]) {
    let status = Command::new(command)
        .args(args)
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
    window::build_window_library();
    window::link_window_library();

    #[cfg(target_os = "macos")]
    {
        macos::download_molten_vk();
        macos::copy_vulkan_files_to_build_path();
        macos::set_rpath();
    }
    #[cfg(all(debug_assertions, feature = "vvl"))]
    {
        vvl::copy_vvl_files_to_deps_dir();
        vvl::generate_config_toml_to_use_vvl();
    }

    println!(
        "cargo:rustc-link-search=native={}",
        deps::get_vulkan_lib_path().display()
    );
    println!("cargo:rerun-if-changed=build.rs");
}
