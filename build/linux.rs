use std::{env, path::Path};

/// ウィンドウライブラリをビルドする関数
///
/// 既にライブラリファイルが存在する場合はスキップする。
pub fn build_window_library() {
    if Path::new("deps/libwindow.a").exists() {
        return;
    }
    super::run("window/linux/build.sh", &[]);
}

pub fn link_window_library() {
    println!(
        "cargo:rustc-link-search=native={}",
        env::current_dir().unwrap().join("deps").display()
    );
    println!("cargo:rustc-link-lib=window");
    println!("cargo:rustc-link-lib=xcb");
    println!("cargo:rustc-link-lib=stdc++");
}
