use std::{env, fs, path::Path};

const VULKAN_LIB_NAME: &str = "libvulkan.1.3.243.dylib";
const VULKAN_LIB_NAME_STEM: &str = "vulkan.1.3.243";
const VULKAN_LIB_INSTALL_NAME: &str = "libvulkan.1.dylib";

pub fn copy_vulkan_dylib(vulkan_lib_path: &str) {
    let src = Path::new(vulkan_lib_path).join(VULKAN_LIB_NAME);
    let dst = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("target")
        .join(env::var("PROFILE").unwrap())
        .join(VULKAN_LIB_INSTALL_NAME);
    if !dst.exists() {
        fs::copy(src, dst).unwrap();
    }
}

pub fn print_link_info() {
    println!("cargo:rustc-link-lib={VULKAN_LIB_NAME_STEM}");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
}
