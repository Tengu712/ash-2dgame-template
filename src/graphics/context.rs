use ash::{vk, *};
use std::{
    ffi::CStr,
    slice,
    sync::{Arc, Mutex},
};

/// Vulkanインスタンスにおける主要オブジェクト群
///
/// スレッドセーフ。
pub struct Context {
    pub entry: Arc<Entry>,
    pub instance: Instance,
    pub physical_device: vk::PhysicalDevice,
    pub queue_family_index: u32,
    pub device: Device,
    pub queue: Mutex<vk::Queue>,
}

impl Context {
    pub fn new(entry: Arc<Entry>, app_name: &CStr, app_version: u32) -> Self {
        let instance = create_instance(&entry, app_name, app_version);
        let physical_device = select_physical_device(&instance);
        let queue_family_index = find_graphics_queue_family_index(&instance, physical_device);
        let device = create_device(&instance, physical_device, queue_family_index);
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let queue = Mutex::new(queue);
        Self {
            entry,
            instance,
            physical_device,
            queue_family_index,
            device,
            queue,
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().expect("failed to wait idle");
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

/// Vulkanインスタンスを作成する関数
///
/// - debugビルド時はバリデーションレイヤーを有効化する。
/// - macOSではMoltenVKのための拡張等を有効化する。
/// - 各OS専用のサーフェス拡張を有効化する。
fn create_instance(entry: &Entry, app_name: &CStr, app_version: u32) -> Instance {
    #[cfg(debug_assertions)]
    let layers = [c"VK_LAYER_KHRONOS_validation".as_ptr()];
    #[cfg(not(debug_assertions))]
    let layers = [];

    #[cfg(target_os = "windows")]
    let extensions = [c"VK_KHR_surface".as_ptr(), c"VK_KHR_win32_surface".as_ptr()];
    #[cfg(target_os = "macos")]
    let extensions = [
        c"VK_KHR_surface".as_ptr(),
        c"VK_KHR_portability_enumeration".as_ptr(),
    ];

    #[cfg(target_os = "macos")]
    let flags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
    #[cfg(not(target_os = "macos"))]
    let flags = vk::InstanceCreateFlags::empty();

    let ai = vk::ApplicationInfo::default()
        .application_name(app_name)
        .application_version(app_version)
        .api_version(vk::make_api_version(0, 1, 0, 0));
    let ci = vk::InstanceCreateInfo::default()
        .flags(flags)
        .application_info(&ai)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions);
    unsafe {
        entry
            .create_instance(&ci, None)
            .expect("failed to create a Vulkan instance")
    }
}

/// 物理デバイスを選択する関数
///
/// discreteかつVRAMの大きいデバイスを優先する。
fn select_physical_device(instance: &Instance) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("failed to enumerate physical devices")
    };
    physical_devices
        .into_iter()
        .max_by_key(|&device| unsafe {
            let props = instance.get_physical_device_properties(device);
            let mem_props = instance.get_physical_device_memory_properties(device);
            let is_discrete = props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;
            let vram = mem_props
                .memory_heaps_as_slice()
                .iter()
                .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
                .map(|heap| heap.size)
                .max()
                .unwrap_or(0);
            (is_discrete, vram)
        })
        .expect("failed to find any physical device")
}

fn find_graphics_queue_family_index(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> u32 {
    unsafe {
        instance
            .get_physical_device_queue_family_properties(physical_device)
            .iter()
            .enumerate()
            .find(|(_, prop)| prop.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .expect("failed to find any graphics queue family index")
            .0 as u32
    }
}

fn create_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
) -> Device {
    let extensions = [c"VK_KHR_swapchain".as_ptr()];
    let qci = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family_index)
        .queue_priorities(&[1.0]);
    let ci = vk::DeviceCreateInfo::default()
        .queue_create_infos(slice::from_ref(&qci))
        .enabled_extension_names(&extensions);
    unsafe {
        instance
            .create_device(physical_device, &ci, None)
            .expect("failed to create a Vulkan device")
    }
}
