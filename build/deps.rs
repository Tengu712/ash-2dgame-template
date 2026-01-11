use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
};

struct Dependency {
    path: &'static str,
    url: &'static str,
    commit: &'static str,
    extra_flags: fn() -> Vec<String>,
}

impl Dependency {
    fn install(&self) {
        if self.stamp_path().exists() {
            return;
        }
        self.clone();
        self.build();
    }

    fn clone(&self) {
        let path = self.path();
        if path.exists() {
            return;
        }

        fs::create_dir_all(&path).unwrap();
        super::run_on(&path, "git", &["init"]);
        super::run_on(&path, "git", &["remote", "add", "origin", self.url]);
        super::run_on(
            &path,
            "git",
            &["fetch", "--depth", "1", "origin", self.commit],
        );
        super::run_on(&path, "git", &["checkout", "FETCH_HEAD"]);
    }

    fn build(&self) {
        let build_path = self.path().join("build");
        if !build_path.exists() {
            fs::create_dir_all(&build_path).unwrap();
        }

        let prefix = format!("-DCMAKE_INSTALL_PREFIX={}", self.install_path().display());
        let base_flags = vec!["..", "-G", "Ninja", "-DCMAKE_BUILD_TYPE=Release", &prefix];
        let extra_flags = (self.extra_flags)();
        let flags = base_flags
            .into_iter()
            .chain(extra_flags.iter().map(|s| s.as_str()))
            .collect::<Vec<_>>();

        super::run_on(&build_path, "cmake", &flags);
        super::run_on(&build_path, "ninja", &[]);
        super::run_on(&build_path, "cmake", &["--install", "."]);
        File::create(self.stamp_path()).unwrap();
    }

    fn path(&self) -> PathBuf {
        env::current_dir().unwrap().join(self.path)
    }

    fn install_path(&self) -> PathBuf {
        self.path().join("install")
    }

    fn stamp_path(&self) -> PathBuf {
        self.install_path().join(".stamp")
    }
}

const VULKAN_HEADERS: Dependency = Dependency {
    path: "deps/Vulkan-Headers",
    url: "https://github.com/KhronosGroup/Vulkan-Headers.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || vec![],
};

const VULKAN_LOADER: Dependency = Dependency {
    path: "deps/Vulkan-Loader",
    url: "https://github.com/KhronosGroup/Vulkan-Loader.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || {
        vec![format!(
            "-DVULKAN_HEADERS_INSTALL_DIR={}",
            path_to_string(&VULKAN_HEADERS.install_path())
        )]
    },
};

const SPIRV_HEADERS: Dependency = Dependency {
    path: "deps/SPIRV-Headers",
    url: "https://github.com/KhronosGroup/SPIRV-Headers.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || vec![],
};

const SPIRV_TOOLS: Dependency = Dependency {
    path: "deps/SPIRV-Tools",
    url: "https://github.com/KhronosGroup/SPIRV-Tools.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || {
        vec![format!(
            "-DSPIRV-Headers_SOURCE_DIR={}",
            path_to_string(&SPIRV_HEADERS.path())
        )]
    },
};

const GLSLANG: Dependency = Dependency {
    path: "deps/glslang",
    url: "https://github.com/KhronosGroup/glslang.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || {
        vec![
            "-DALLOW_EXTERNAL_SPIRV_TOOLS=ON".to_string(),
            format!(
                "-DCMAKE_PREFIX_PATH={}",
                path_to_string(&SPIRV_TOOLS.install_path())
            ),
        ]
    },
};

#[cfg(all(debug_assertions, feature = "vvl"))]
const VULKAN_UTILITY_LIBRARIES: Dependency = Dependency {
    path: "deps/Vulkan-Utility-Libraries",
    url: "https://github.com/KhronosGroup/Vulkan-Utility-Libraries.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || {
        vec![format!(
            "-DVULKAN_HEADERS_INSTALL_DIR={}",
            path_to_string(&VULKAN_HEADERS.install_path())
        )]
    },
};

#[cfg(all(debug_assertions, feature = "vvl"))]
const VULKAN_VALIDATION_LAYERS: Dependency = Dependency {
    path: "deps/Vulkan-ValidationLayers",
    url: "https://github.com/KhronosGroup/Vulkan-ValidationLayers.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || {
        vec![
            format!(
                "-DGLSLANG_INSTALL_DIR={}",
                path_to_string(&GLSLANG.install_path())
            ),
            format!(
                "-DSPIRV_HEADERS_INSTALL_DIR={}",
                path_to_string(&SPIRV_HEADERS.install_path())
            ),
            format!(
                "-DSPIRV_TOOLS_INSTALL_DIR={}",
                path_to_string(&SPIRV_TOOLS.install_path())
            ),
            format!(
                "-DVULKAN_HEADERS_INSTALL_DIR={}",
                path_to_string(&VULKAN_HEADERS.install_path())
            ),
            format!(
                "-DVULKAN_UTILITY_LIBRARIES_INSTALL_DIR={}",
                path_to_string(&VULKAN_UTILITY_LIBRARIES.install_path())
            ),
        ]
    },
};

pub fn install_dependencies() {
    VULKAN_HEADERS.install();
    VULKAN_LOADER.install();
    SPIRV_HEADERS.install();
    SPIRV_TOOLS.install();
    GLSLANG.install();
    #[cfg(all(debug_assertions, feature = "vvl"))]
    {
        VULKAN_UTILITY_LIBRARIES.install();
        VULKAN_VALIDATION_LAYERS.install();
    }
}

pub fn get_vulkan_lib_path() -> String {
    path_to_string(&VULKAN_LOADER.install_path().join("lib"))
}

pub fn get_glslang_path() -> String {
    path_to_string(&GLSLANG.install_path().join("bin").join("glslang"))
}

#[cfg(all(debug_assertions, feature = "vvl"))]
pub fn get_vvl_path() -> String {
    path_to_string(&VULKAN_VALIDATION_LAYERS.install_path())
}

fn path_to_string(path: &Path) -> String {
    path.as_os_str().display().to_string().replace("\\", "/")
}
