use std::{
    fs::{self, File},
    path::PathBuf,
};

struct Dependency {
    name: &'static str,
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
        super::deps_path().join(self.name)
    }

    fn install_path(&self) -> PathBuf {
        self.path().join("install")
    }

    fn stamp_path(&self) -> PathBuf {
        self.install_path().join(".stamp")
    }
}

const VULKAN_HEADERS: Dependency = Dependency {
    name: "Vulkan-Headers",
    url: "https://github.com/KhronosGroup/Vulkan-Headers.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || vec![],
};

const VULKAN_LOADER: Dependency = Dependency {
    name: "Vulkan-Loader",
    url: "https://github.com/KhronosGroup/Vulkan-Loader.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || {
        vec![format!(
            "-DVULKAN_HEADERS_INSTALL_DIR={}",
            VULKAN_HEADERS.install_path().display()
        )]
    },
};

const SPIRV_HEADERS: Dependency = Dependency {
    name: "SPIRV-Headers",
    url: "https://github.com/KhronosGroup/SPIRV-Headers.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || vec![],
};

const SPIRV_TOOLS: Dependency = Dependency {
    name: "SPIRV-Tools",
    url: "https://github.com/KhronosGroup/SPIRV-Tools.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || {
        vec![format!(
            "-DSPIRV-Headers_SOURCE_DIR={}",
            SPIRV_HEADERS.path().display()
        )]
    },
};

const GLSLANG: Dependency = Dependency {
    name: "glslang",
    url: "https://github.com/KhronosGroup/glslang.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || {
        vec![
            "-DALLOW_EXTERNAL_SPIRV_TOOLS=ON".to_string(),
            format!(
                "-DCMAKE_PREFIX_PATH={}",
                SPIRV_TOOLS.install_path().display()
            ),
        ]
    },
};

#[cfg(all(debug_assertions, feature = "vvl"))]
const VULKAN_UTILITY_LIBRARIES: Dependency = Dependency {
    name: "Vulkan-Utility-Libraries",
    url: "https://github.com/KhronosGroup/Vulkan-Utility-Libraries.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || {
        vec![format!(
            "-DVULKAN_HEADERS_INSTALL_DIR={}",
            VULKAN_HEADERS.install_path().display()
        )]
    },
};

#[cfg(all(debug_assertions, feature = "vvl"))]
const VULKAN_VALIDATION_LAYERS: Dependency = Dependency {
    name: "Vulkan-ValidationLayers",
    url: "https://github.com/KhronosGroup/Vulkan-ValidationLayers.git",
    commit: "vulkan-sdk-1.4.335.0",
    extra_flags: || {
        vec![
            format!("-DGLSLANG_INSTALL_DIR={}", GLSLANG.install_path().display()),
            format!(
                "-DSPIRV_HEADERS_INSTALL_DIR={}",
                SPIRV_HEADERS.install_path().display()
            ),
            format!(
                "-DSPIRV_TOOLS_INSTALL_DIR={}",
                SPIRV_TOOLS.install_path().display()
            ),
            format!(
                "-DVULKAN_HEADERS_INSTALL_DIR={}",
                VULKAN_HEADERS.install_path().display()
            ),
            format!(
                "-DVULKAN_UTILITY_LIBRARIES_INSTALL_DIR={}",
                VULKAN_UTILITY_LIBRARIES.install_path().display()
            ),
            #[cfg(target_os = "linux")]
            "-DBUILD_WSI_XLIB_SUPPORT=OFF".to_string(),
            #[cfg(target_os = "linux")]
            "-DBUILD_WSI_WAYLAND_SUPPORT=OFF".to_string(),
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

pub fn get_vulkan_lib_path() -> PathBuf {
    VULKAN_LOADER.install_path().join("lib")
}

pub fn get_glslang_path() -> PathBuf {
    GLSLANG.install_path().join("bin").join("glslang")
}

#[cfg(all(debug_assertions, feature = "vvl"))]
pub fn get_vvl_path() -> PathBuf {
    VULKAN_VALIDATION_LAYERS.install_path()
}
