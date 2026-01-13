use super::deps;
use std::{fs, path::Path};

/// VVLに関連するファイル群をdepsにコピーする関数
///
/// コピー対象:
/// - 共有ライブラリファイル
/// - レイヤーを表すJSONファイル
///
/// NOTE: 特にmacOSとLinuxにおいて、
///       JSONは自身と同じディレクトリにライブラリがあることを期待する。
///       しかし、なぜか別のディレクトリにインストールされるので、
///       まとめてdepsディレクトリにコピーすることにしている。
pub fn copy_vvl_files_to_deps_dir() {
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
    #[cfg(target_os = "linux")]
    const PATHS: [&[&str]; 2] = [
        &["lib", "libVkLayer_khronos_validation.so"],
        &[
            "share",
            "vulkan",
            "explicit_layer.d",
            "VkLayer_khronos_validation.json",
        ],
    ];

    let src = deps::get_vvl_path();
    let dst = super::deps_path();

    for path in PATHS.iter() {
        let src = path.iter().fold(src.to_path_buf(), |acc, p| acc.join(p));
        let dst = dst.join(path.last().unwrap());
        if !dst.exists() {
            fs::copy(src, dst).unwrap();
        }
    }
}

/// VVLを有効化するために必要な実行時環境変数を指定するTOMLファイルを生成する関数
///
/// NOTE: MoltenVKと違ってVVLへのパスはプロセス起動時に通されていなければならない。
///       そのため、.cargo/config.tomlに書いておくことで、解決する。
///       どうせVVLは開発時にしか必要としないので問題ないと考えられる。
///
/// WARN: このTOMLファイルがない状態で`cargo run --features vvl`すると、
///       TOMLファイルは生成されるがその内容は反映されない。
///       そのため2度`run`するか、一度`build`してから`run`すること。
pub fn generate_config_toml_to_use_vvl() {
    let path = super::deps_path()
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
