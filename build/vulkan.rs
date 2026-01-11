use super::deps;

pub fn add_vulkan_lib_path_to_libpath() {
    println!(
        "cargo:rustc-link-search=native={}",
        deps::get_vulkan_lib_path()
    );
}

#[cfg(all(debug_assertions, feature = "vvl"))]
pub fn copy_vvl_files_to_deps_dir() {
    use std::{env, fs, path::Path};

    #[cfg(target_os = "windows")]
    const PATHS: [&[&str]; 2] = [
        &["bin", "VkLayer_khronos_validation.dll"],
        &["bin", "VkLayer_khronos_validation.json"],
    ];
    #[cfg(target_os = "macos")]
    const PATHS: [&[&str]; 2] = [
        &["lib", "libVkLayer_khronos_validation.dylib"],
        &[
            "share",
            "vulkan",
            "explicit_layer.d",
            "VkLayer_khronos_validation.json",
        ],
    ];

    let src = deps::get_vvl_path();
    let src = Path::new(&src);
    let dst = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("deps");

    for path in PATHS.iter() {
        let src = path.iter().fold(src.to_path_buf(), |acc, p| acc.join(p));
        let dst = dst.join(path.last().unwrap());
        if !dst.exists() {
            println!("{} {}", src.display(), dst.display());
            fs::copy(src, dst).unwrap();
        }
    }
}

#[cfg(all(debug_assertions, feature = "vvl"))]
pub fn generate_config_toml_to_use_vvl() {
    use std::{env, fs, path::Path};

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("deps")
        .display()
        .to_string()
        .replace("\\", "\\\\");
    let cargo_config = format!(
        "\
[env]
VK_LAYER_PATH = {{ value = \"{path}\", force = true }}
DYLD_LIBRARY_PATH = {{ value = \"{path}\", force = true }}\
    "
    );
    fs::create_dir_all(".cargo").unwrap();
    fs::write(".cargo/config.toml", cargo_config).unwrap();
}
