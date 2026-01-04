use std::{
    env,
    fs::{self, File},
    path::Path,
};

const CONAN_PROFILE_COMMANDS: &[&str] = &["conan", "profile", "detect", "--exist-ok"];
const CONAN_INSTALL_COMMANDS: &[&str] = &[
    "conan",
    "install",
    ".",
    "--output-folder",
    "./deps",
    "--build=missing",
    "-s",
    "build_type=Release",
    "-s",
    "compiler.cppstd=17",
    "-c",
    "tools.cmake.cmaketoolchain:generator=Ninja",
];

/// Conan2で依存パッケージをインストールする関数
///
/// conanがインストールされている場合conanを、
/// そうでない場合uv tool run conanを実行する。
///
/// deps/.stampが存在する場合はスキップする。
pub fn install_dependencies() {
    use super::{is_command_available, run};

    let path = Path::new("deps/.stamp");
    if path.exists() {
        return;
    }

    if is_command_available("conan") {
        run(CONAN_PROFILE_COMMANDS[0], &CONAN_PROFILE_COMMANDS[1..]);
        run(CONAN_INSTALL_COMMANDS[0], &CONAN_INSTALL_COMMANDS[1..]);
    } else if is_command_available("uv") {
        run("uv", &[&["tool", "run"], CONAN_PROFILE_COMMANDS].concat());
        run("uv", &[&["tool", "run"], CONAN_INSTALL_COMMANDS].concat());
    } else {
        panic!("you must install conan or uv for installing dependencies.");
    }
    File::create(path).unwrap();

    println!("cargo:rerun-if-changed=conanfile.py");
}

/// デバッグビルド時に.cargo/config.tomlを作成する関数
///
/// Vulkan Validation Layersを追加するためには実行時に環境変数を指定する必要がある。
/// cargo runでの実行時に限れば、.cargo/config.tomlを使って実現できる。
///
/// Vulkan Validation Layersに必要なファイルはconanfile.pyでdepsディレクトリにコピーされる。
/// ![参照](../conanfile.py)
///
/// リリースビルド時は削除する。
pub fn generate_cargo_config() {
    if env::var("PROFILE").unwrap() != "debug" {
        let _ = fs::remove_file(".cargo/config.toml");
        return;
    }

    let deps_path = Path::new(&env::current_dir().unwrap()).join("deps");
    let cargo_config = format!(
        r#"[env]
VK_LAYER_PATH = {{ value = "{0}", force = true }}
DYLD_LIBRARY_PATH = {{ value = "{0}", force = true }}
"#,
        deps_path.display().to_string().replace('\\', "/"),
    );

    fs::create_dir_all(".cargo").unwrap();
    fs::write(".cargo/config.toml", cargo_config).unwrap();
}

pub fn add_vulkan_lib_dirpath_to_libpath() {
    println!(
        "cargo:rustc-link-search=native={}",
        get_vulkan_lib_dirpath()
    );
}

pub fn get_vulkan_lib_dirpath() -> String {
    get_config("VULKAN_LIB")
}

pub fn get_glslang_bin_dirpath() -> String {
    get_config("GLSLANG_BIN")
}

fn get_config(key: &str) -> String {
    let content = fs::read_to_string("deps/conan-paths.txt").unwrap();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (k, v) = line.split_once('=').unwrap();
        if k.trim() == key {
            return v.trim().to_string();
        }
    }
    panic!("{key} not found in deps/conan-paths.txt");
}
