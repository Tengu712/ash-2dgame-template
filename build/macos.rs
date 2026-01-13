use super::deps;
use std::fs;

/// MoltenVKをダウンロードする関数
///
/// NOTE: depsモジュールにある依存とは異なり、
///       自前ビルドするのが相当面倒くさそうな上、
///       GitHub上で事前ビルドされたバイナリが公開されているため、
///       それをダウンロードして用いる。
pub fn download_molten_vk() {
    const URL: &str =
        "https://github.com/KhronosGroup/MoltenVK/releases/download/v1.4.1/MoltenVK-macos.tar";
    const TAR: &str = "MoltenVK-macos.tar";
    const DIR: &str = "MoltenVK-macos";

    let deps_path = super::deps_path();

    if !deps_path.join(TAR).exists() {
        super::run_on(&deps_path, "curl", &["-4", "-o", TAR, "-L", URL]);
    }
    if !deps_path.join(DIR).exists() {
        super::run_on(&deps_path, "mkdir", &[DIR]);
        super::run_on(&deps_path, "tar", &["-xf", TAR, "-C", DIR]);
    }
}

/// ビルドディレクトリにVulkanを利用するために必要なファイル群をコピーする関数
///
/// コピー対象:
/// - Vulkanローダ
/// - MoltenVK (ライブラリ)
/// - MoltenVK (ICD)
///
/// NOTE: 他OSとは違ってmacOSはVulkanをサポートしていない。
///       従って、VulkanローダとMoltenVKを同梱する必要がある。
pub fn copy_vulkan_files_to_build_path() {
    const VULKAN_LIB_NAME: &str = "libvulkan.1.4.335.dylib";
    const VULKAN_LIB_INSTALL_NAME: &str = "libvulkan.1.dylib";
    const MOLTEN_VK_DYLIB_NAME: &str = "libMoltenVK.dylib";
    const MOLTEN_VK_JSON_NAME: &str = "MoltenVK_icd.json";

    let vulkan_lib_path = deps::get_vulkan_lib_path();
    let molten_vk_path = super::deps_path()
        .join("MoltenVK-macos")
        .join("MoltenVK")
        .join("MoltenVK")
        .join("dynamic")
        .join("dylib")
        .join("macOS");
    let sd_sn_dn = [
        (&vulkan_lib_path, VULKAN_LIB_NAME, VULKAN_LIB_INSTALL_NAME),
        (&molten_vk_path, MOLTEN_VK_DYLIB_NAME, MOLTEN_VK_DYLIB_NAME),
        (&molten_vk_path, MOLTEN_VK_JSON_NAME, MOLTEN_VK_JSON_NAME),
    ];

    for (sd, sn, dn) in sd_sn_dn.iter() {
        let dst = super::build_path().join(dn);
        if !dst.exists() {
            fs::copy(sd.join(sn), dst).unwrap();
        }
    }
}

pub fn set_rpath() {
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
}
