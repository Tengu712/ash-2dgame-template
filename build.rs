use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
};

const VULKAN_HEADERS_DEP: Dependency = Dependency {
    path: "deps/Vulkan-Headers",
    url: "https://github.com/KhronosGroup/Vulkan-Headers.git",
    sha: "450bd2232225d6c7728a4108055ac2e37cef6475",
};
const VULKAN_LOADER_DEP: Dependency = Dependency {
    path: "deps/Vulkan-Loader",
    url: "https://github.com/KhronosGroup/Vulkan-Loader.git",
    sha: "7a07afe04ad134d4eabe25f62720177f60ed6627",
};

struct Dependency {
    path: &'static str,
    url: &'static str,
    sha: &'static str,
}

impl Dependency {
    fn get_path(&self) -> PathBuf {
        env::current_dir().unwrap().join(self.path)
    }

    fn get_install_path(&self) -> PathBuf {
        self.get_path().join("installed")
    }

    fn clone_if_needed(&self) {
        let path = self.get_path();
        if path.exists() {
            return;
        }

        fs::create_dir_all(&path).unwrap();
        run(&path, "git", &["init"]);
        run(&path, "git", &["remote", "add", "origin", self.url]);
        run(&path, "git", &["fetch", "--depth", "1", "origin", self.sha]);
        run(&path, "git", &["checkout", "FETCH_HEAD"]);
    }

    fn build_if_needed(&self, cmake_add_flags: &[&str]) {
        let path = self.get_path();
        let install_path = self.get_install_path();
        let stamp_path = install_path.join(".stamp");
        if stamp_path.exists() {
            return;
        }

        let build_path = path.join("build");
        if !build_path.exists() {
            fs::create_dir_all(&build_path).unwrap();
        }

        let install_prefix = format!("-DCMAKE_INSTALL_PREFIX={}", install_path.display());
        let mut build_flags = vec![
            "..",
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            &install_prefix,
        ];
        build_flags.extend_from_slice(cmake_add_flags);

        run(&build_path, "cmake", &build_flags);
        run(
            &build_path,
            "cmake",
            &["--build", ".", "--config", "Release"],
        );
        run(&build_path, "cmake", &["--install", "."]);
        File::create(stamp_path).unwrap();
    }
}

fn run(current_dir: &Path, command: &str, args: &[&str]) {
    let status = Command::new(command)
        .current_dir(current_dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success());
}

fn print_link_info() {
    let lib_path = VULKAN_LOADER_DEP.get_install_path().join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    #[cfg(windows)]
    println!("cargo:rustc-link-lib=vulkan-1");
    #[cfg(not(windows))]
    println!("cargo:rustc-link-lib=vulkan");
}

fn main() {
    VULKAN_HEADERS_DEP.clone_if_needed();
    VULKAN_LOADER_DEP.clone_if_needed();
    VULKAN_HEADERS_DEP.build_if_needed(&[]);
    VULKAN_LOADER_DEP.build_if_needed(&[&format!(
        "-DVULKAN_HEADERS_INSTALL_DIR={}",
        VULKAN_HEADERS_DEP.get_install_path().display()
    )]);
    print_link_info();
    println!("cargo:rerun-if-changed=build.rs");
}
