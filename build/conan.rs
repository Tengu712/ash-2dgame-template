use std::{
    collections::HashMap,
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

pub fn print_link_info() {
    let config = parse_config();
    println!(
        "cargo:rustc-link-search=native={}",
        config.get("VULKAN_LIB").unwrap()
    );
}

pub fn parse_config() -> HashMap<String, String> {
    fs::read_to_string("deps/conan-paths.txt")
        .unwrap()
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            line.split_once('=')
                .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        })
        .collect()
}
