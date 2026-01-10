use super::deps;

pub fn add_vulkan_lib_path_to_libpath() {
    println!(
        "cargo:rustc-link-search=native={}",
        deps::get_vulkan_lib_path()
    );
}

#[cfg(all(debug_assertions, feature = "vvl"))]
pub fn generate_config_toml_to_use_vvl() {
    use std::fs;

    let cargo_config = format!(
        "\
[env]
VK_LAYER_PATH = {{ value = \"{0}\", force = true }}
DYLD_LIBRARY_PATH = {{ value = \"{0}\", force = true }}\
    ",
        deps::get_vvl_path()
    );
    fs::create_dir_all(".cargo").unwrap();
    fs::write(".cargo/config.toml", cargo_config).unwrap();
}
