use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio},
};

const INSTALL_DEPS_CMAKE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/install_deps.cmake");
const DEPS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/deps");
const VULKAN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/deps/Vulkan-Loader-vulkan-sdk-1.4.335.0/install/lib"
);
const GLSLANG: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/deps/glslang-vulkan-sdk-1.4.335.0/install/bin/glslang"
);

fn run(command: &str, args: &[&str]) {
    let status = Command::new(command)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();
    assert!(status.success());
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=deps/");
    println!("cargo:rerun-if-changed=shader/");
    println!("cargo:rerun-if-changed=window/");
    println!("cargo:rustc-link-search=native={DEPS}");
    println!("cargo:rustc-link-search=native={VULKAN}");

    let cargo_build_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(env::var("PROFILE").unwrap());
    let cargo_build_path = format!("-DCARGO_BUILD_PATH={}", cargo_build_path.display());
    let args = [
        #[cfg(all(debug_assertions, feature = "vvl"))]
        "-DVVL=ON",
        &cargo_build_path,
        "-P",
        &INSTALL_DEPS_CMAKE,
    ];
    run("cmake", &args);

    let shaders = ["shader/basic.frag", "shader/basic.vert"];
    for shader in shaders {
        run(GLSLANG, &["-V", shader, "-o", &format!("{shader}.spv")]);
    }

    #[cfg(target_os = "windows")]
    let libs = ["window", "gdi32", "user32", "vulkan-1"];
    #[cfg(target_os = "macos")]
    let libs = ["window", "framework=Cocoa", "framework=QuartzCore"];
    #[cfg(target_os = "linux")]
    let libs = ["window", "xcb", "stdc++"];
    for lib in libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
    }
}
