use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    path::Path,
    process::{Command, Stdio},
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

#[cfg(target_os = "macos")]
const VULKAN_LIB_NAME: &str = "libvulkan.1.3.243.dylib";
#[cfg(target_os = "macos")]
const VULKAN_LIB_NAME_STEM: &str = "vulkan.1.3.243";
#[cfg(target_os = "macos")]
const VULKAN_LIB_INSTALL_NAME: &str = "libvulkan.1.dylib";

fn is_command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run(command: &str, args: &[&str]) {
    let status = Command::new(command)
        .args(args)
        .env("CARGO_PROFILE", env::var("PROFILE").unwrap())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();
    assert!(status.success());
}

fn install_dependencies() {
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
}

fn parse_config() -> HashMap<String, String> {
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

fn print_link_info() {
    let config = parse_config();
    println!(
        "cargo:rustc-link-search=native={}",
        config.get("VULKAN_LIB").unwrap()
    );
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=vulkan-1");
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib={VULKAN_LIB_NAME_STEM}");
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=vulkan");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
    }
}

fn copy_vulkan_dylib() {
    #[cfg(target_os = "macos")]
    {
        let config = parse_config();
        let src = Path::new(config.get("VULKAN_LIB").unwrap()).join(VULKAN_LIB_NAME);
        let dst = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("target")
            .join(env::var("PROFILE").unwrap())
            .join(VULKAN_LIB_INSTALL_NAME);
        println!("src={}", src.display());
        println!("dst={}", dst.display());
        if !dst.exists() {
            fs::copy(src, dst).unwrap();
        }
    }
}

fn generate_cargo_config() {
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
        deps_path.display(),
    );

    fs::create_dir_all(".cargo").unwrap();
    fs::write(".cargo/config.toml", cargo_config).unwrap();
}

fn main() {
    install_dependencies();
    print_link_info();
    copy_vulkan_dylib();
    generate_cargo_config();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=conanfile.py");
}
