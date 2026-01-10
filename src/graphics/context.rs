use crate::logs::*;
use ash::{
    khr::{self, surface, swapchain},
    prelude::VkResult,
    vk, *,
};
use std::{ffi::CStr, slice, sync::LazyLock};

/// Vulkanローダへのハンドルのようなもの
///
/// NOTE: 次が期待されるためここに`LazyLock`で定義されている:
///       - 1アプリケーション1個である
///       - contextモジュールのみ扱うものである
static ENTRY: LazyLock<Entry> = LazyLock::new(Entry::linked);

/// Vulkanインスタンスにおける主要オブジェクト群
///
/// 一個のVulkanインスタンスを表す。
/// 複数個作ることはできるが、マルチスレッド対応していないこともあって、メリットはない。
///
/// WARN: マルチスレッド対応していないため、単一スレッドでのみ扱うこと。
//
// HINT: マルチスレッド対応する場合は、`queueを`Mutex<vk::Queue>`にすると良い。
pub struct Context {
    pub instance: Instance,
    pub physical_device: vk::PhysicalDevice,
    pub queue_family_index: u32,
    pub device: Device,
    pub queue: vk::Queue,

    pub surface_loader: surface::Instance,
    pub swapchain_loader: swapchain::Device,

    #[cfg(target_os = "windows")]
    pub win32_surface_loader: khr::win32_surface::Instance,
}

impl Context {
    pub fn new(app_name: &CStr, app_version: u32) -> Self {
        let instance = create_instance(&ENTRY, app_name, app_version);
        let physical_device = select_physical_device(&instance);
        let queue_family_index = find_graphics_queue_family_index(&instance, physical_device);
        let device = create_device(&instance, physical_device, queue_family_index);
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        let surface_loader = surface::Instance::new(&ENTRY, &instance);
        let swapchain_loader = swapchain::Device::new(&instance, &device);

        #[cfg(target_os = "windows")]
        let win32_surface_loader = khr::win32_surface::Instance::new(&ENTRY, &instance);

        Self {
            instance,
            physical_device,
            queue_family_index,
            device,
            queue,
            surface_loader,
            swapchain_loader,
            #[cfg(target_os = "windows")]
            win32_surface_loader,
        }
    }

    pub fn wait_idle(&self) -> VkResult<()> {
        unsafe { self.device.device_wait_idle() }
    }
}

impl Context {
    pub fn find_memory_type_index(
        &self,
        type_bits: u32,
        mask: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        let props = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };
        props.memory_types[..props.memory_type_count as _]
            .iter()
            .enumerate()
            .find(|(i, mtype)| (1 << i) & type_bits != 0 && mtype.property_flags & mask == mask)
            .map(|(i, _)| i as _)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
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
    let layers = [
        #[cfg(all(debug_assertions, feature = "vvl"))]
        c"VK_LAYER_KHRONOS_validation".as_ptr(),
    ];
    let extensions = [
        c"VK_KHR_surface".as_ptr(),
        #[cfg(target_os = "windows")]
        c"VK_KHR_win32_surface".as_ptr(),
        #[cfg(target_os = "macos")]
        c"VK_KHR_portability_enumeration".as_ptr(),
    ];

    #[cfg(target_os = "macos")]
    let flags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
    #[cfg(not(target_os = "macos"))]
    let flags = vk::InstanceCreateFlags::empty();

    let ai = vk::ApplicationInfo::default()
        .application_name(app_name)
        .application_version(app_version)
        .api_version(vk::make_api_version(0, 1, 1, 0));
    let ci = vk::InstanceCreateInfo::default()
        .flags(flags)
        .application_info(&ai)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions);
    unsafe {
        entry
            .create_instance(&ci, None)
            .expect_log("failed to create a Vulkan instance")
    }
}

/// 物理デバイスを選択する関数
///
/// discreteかつVRAMの大きいデバイスを優先する。
fn select_physical_device(instance: &Instance) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect_log("failed to enumerate physical devices")
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
        .expect_log("failed to find any physical device")
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
            .expect_log("failed to find any graphics queue family index")
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
            .expect_log("failed to create a Vulkan device")
    }
}
