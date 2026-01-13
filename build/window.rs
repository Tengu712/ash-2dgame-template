struct Info {
    script: &'static str,
    name: &'static str,
    libs: &'static [&'static str],
}

#[cfg(target_os = "windows")]
const INFO: Info = Info {
    script: "window\\windows\\build.bat",
    name: "window.lib",
    libs: &["window", "gdi32", "user32", "vulkan-1"],
};

#[cfg(target_os = "macos")]
const INFO: Info = Info {
    script: "window/macos/build.sh",
    name: "libwindow.a",
    libs: &["window", "framework=Cocoa", "framework=QuartzCore"],
};

#[cfg(target_os = "linux")]
const INFO: Info = Info {
    script: "window/linux/build.sh",
    name: "libwindow.a",
    libs: &["window", "xcb", "stdc++"],
};

pub fn build_window_library() {
    if super::deps_path().join(INFO.name).exists() {
        return;
    }
    super::run(INFO.script, &[]);
}

pub fn link_window_library() {
    println!(
        "cargo:rustc-link-search=native={}",
        super::deps_path().display()
    );
    for lib in INFO.libs.iter() {
        println!("cargo:rustc-link-lib={lib}");
    }
}
