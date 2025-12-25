use ash::{Entry, Instance, vk};
use std::ffi::CStr;

/// Vulkanインスタンスにおける主要オブジェクト群
///
/// NOTE: Vulkanインスタンスは1アプリケーション上で複数個作成できる。
///       そのため、VulkanLoaderへの参照の表現であるash::Entryは含まない。
pub struct Context {
    pub instance: Instance,
}

impl Context {
    pub fn new(entry: &Entry, app_name: &CStr, app_version: u32) -> Self {
        let instance = create_instance(entry, app_name, app_version);
        Self { instance }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

fn create_instance(entry: &Entry, app_name: &CStr, app_version: u32) -> Instance {
    #[cfg(debug_assertions)]
    let layers = [c"VK_LAYER_KHRONOS_validation".as_ptr()];
    #[cfg(not(debug_assertions))]
    let layers = [];
    let ai = vk::ApplicationInfo::default()
        .application_name(app_name)
        .application_version(app_version)
        .api_version(vk::make_api_version(0, 1, 0, 0));
    let ci = vk::InstanceCreateInfo::default()
        .application_info(&ai)
        .enabled_layer_names(&layers);
    unsafe {
        entry
            .create_instance(&ci, None)
            .expect("failed to create a Vulkan instance")
    }
}
