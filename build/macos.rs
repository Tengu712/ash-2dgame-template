use super::deps;
use std::{env, fs, path::Path};

/// ウィンドウライブラリをビルドする関数
///
/// 既にライブラリファイルが存在する場合はスキップする。
///
/// NOTE: ウィンドウライブラリを更新する場合、deps/libwindow.aを削除してリビルドすること。
pub fn build_window_library() {
    if !Path::new("deps/libwindow.a").exists() {
        super::run("window/macos/build.sh", &[]);
    }

    println!("cargo:rerun-if-changed=window/macos/build.sh");
    println!("cargo:rerun-if-changed=window/macos/window.swift");
}

pub fn link_window_library() {
    println!(
        "cargo:rustc-link-search=native={}",
        env::current_dir().unwrap().join("deps").display()
    );
    println!("cargo:rustc-link-lib=window");
    println!("cargo:rustc-link-lib=framework=Cocoa");
    println!("cargo:rustc-link-lib=framework=QuartzCore");
}

/// ビルドディレクトリにlibvulkan.1.dylibをコピーする関数
///
/// NOTE: 他OSとは違って普通macOSはVulkanローダを持っていない。
///       従って、Vulkanローダを同梱する必要がある。
pub fn copy_vulkan_dylib() {
    const VULKAN_LIB_NAME: &str = "libvulkan.1.4.335.dylib";
    const VULKAN_LIB_INSTALL_NAME: &str = "libvulkan.1.dylib";

    let src = Path::new(&deps::get_vulkan_lib_path()).join(VULKAN_LIB_NAME);
    let dst = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("target")
        .join(env::var("PROFILE").unwrap())
        .join(VULKAN_LIB_INSTALL_NAME);
    if !dst.exists() {
        fs::copy(src, dst).unwrap();
    }
}

pub fn copy_molten_vk() {
    const DYLIB_NAME: &str = "libMoltenVK.dylib";
    const JSON_NAME: &str = "MoltenVK_icd.json";

    let src = deps::get_molten_vk_path();
    let src = Path::new(&src);

    for n in [DYLIB_NAME, JSON_NAME].iter() {
        let src = src.join(n);
        let dst = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("target")
            .join(env::var("PROFILE").unwrap())
            .join(n);
        if !dst.exists() {
            fs::copy(src, dst).unwrap();
        }
    }
}

pub fn set_rpath() {
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
}
