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
        unsafe {
            let instance = {
                let ai = vk::ApplicationInfo {
                    p_application_name: app_name.as_ptr(),
                    application_version: app_version,
                    api_version: vk::make_api_version(0, 1, 0, 0),
                    ..Default::default()
                };
                let ci = vk::InstanceCreateInfo {
                    p_application_info: &ai,
                    ..Default::default()
                };
                entry
                    .create_instance(&ci, None)
                    .expect("failed to create a Vulkan instance.")
            };

            Self { instance }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
