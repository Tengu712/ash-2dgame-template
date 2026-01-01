use std::{
    env, fs,
    path::Path,
    process::{Command, Stdio},
};

mod conan;
#[cfg(target_os = "macos")]
mod macos;
mod shader;
#[cfg(target_os = "windows")]
mod windows;

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
        deps_path.display().to_string().replace('\\', "/"),
    );

    fs::create_dir_all(".cargo").unwrap();
    fs::write(".cargo/config.toml", cargo_config).unwrap();
}

fn main() {
    conan::install_dependencies();
    conan::print_link_info();

    let config = conan::parse_config();

    shader::compile_shader(config.get("GLSLANG_BIN").unwrap());

    #[cfg(target_os = "windows")]
    {
        windows::build_window_library();
        windows::print_link_info();
    }
    #[cfg(target_os = "macos")]
    {
        // TODO: build_window_library()
        macos::copy_vulkan_dylib(config.get("VULKAN_LIB").unwrap());
        macos::print_link_info();
    }

    generate_cargo_config();

    println!("cargo:rerun-if-changed=build.rs");
}
