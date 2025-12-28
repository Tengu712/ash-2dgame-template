use ash::{Entry, Instance, khr::win32_surface, vk};
use std::{ffi::c_void, iter};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct WindowSize {
    width: u32,
    height: u32,
}

#[link(name = "window", kind = "static")]
unsafe extern "C" {
    fn get_instance() -> *mut c_void;
    fn create_window(title: *const u16, width: u32, height: u32) -> *mut c_void;
    fn destroy_window(window: *mut c_void);
    fn process_window_events() -> u8;
    fn get_current_client_size(window: *mut c_void) -> WindowSize;
}

pub struct Window {
    instance: *mut c_void,
    window: *mut c_void,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        let instance = unsafe { get_instance() };
        if instance.is_null() {
            panic!("failed to get a instance handle");
        }

        let title = title
            .encode_utf16()
            .chain(iter::once(0))
            .collect::<Vec<_>>();
        let window = unsafe { create_window(title.as_ptr(), width, height) };
        if window.is_null() {
            panic!("failed to create a window");
        }

        Self { instance, window }
    }

    pub fn process_events(&self) -> bool {
        unsafe { process_window_events() != 0 }
    }

    pub fn create_surface(&self, entry: &Entry, instance: &Instance) -> vk::SurfaceKHR {
        let instance = win32_surface::Instance::new(entry, instance);
        let ci = vk::Win32SurfaceCreateInfoKHR::default()
            .hinstance(self.instance as isize)
            .hwnd(self.window as isize);
        unsafe {
            instance
                .create_win32_surface(&ci, None)
                .expect("failed to create a surface")
        }
    }

    pub fn get_current_client_size(&self) -> vk::Extent2D {
        unsafe {
            let size = get_current_client_size(self.window);
            vk::Extent2D {
                width: size.width,
                height: size.height,
            }
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { destroy_window(self.window) };
    }
}
