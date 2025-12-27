use super::run;
use std::{env, path::Path};

/// ウィンドウライブラリをビルドする関数
///
/// 既にライブラリファイルが存在する場合はスキップする。
pub fn build_window_library() {
    if !Path::new("deps/window.lib").exists() {
        run("window\\windows\\build.bat", &[]);
    }

    println!("cargo:rerun-if-changed=window/windows/build.bat");
    println!("cargo:rerun-if-changed=window/windows/window.cpp");
}

pub fn print_link_info() {
    println!(
        "cargo:rustc-link-search=native={}",
        env::current_dir().unwrap().join("deps").display()
    );
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=window");
    println!("cargo:rustc-link-lib=vulkan-1");
}
