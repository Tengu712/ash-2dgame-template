use std::{env, path::Path};

/// ウィンドウライブラリをビルドする関数
///
/// 既にライブラリファイルが存在する場合はスキップする。
///
/// NOTE: ウィンドウライブラリを更新する場合、次の手順を踏むこと:
///       1. deps/window.libを削除
///       2. window.cppあるいはbuild.batを更新
pub fn build_window_library() {
    if !Path::new("deps/window.lib").exists() {
        super::run("window\\windows\\build.bat", &[]);
    }

    println!("cargo:rerun-if-changed=window/windows/build.bat");
    println!("cargo:rerun-if-changed=window/windows/window.cpp");
}

pub fn link_window_library() {
    println!(
        "cargo:rustc-link-search=native={}",
        env::current_dir().unwrap().join("deps").display()
    );
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=window");
    println!("cargo:rustc-link-lib=vulkan-1");
}
