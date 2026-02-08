use crate::logs::*;
use ash::{khr::win32_surface::Instance, prelude::VkResult, vk};
use std::{ffi::c_void, iter, marker::PhantomData};

/// Win32製ウィンドウ
//
// NOTE: ライブラリ側が単一スレッド上の動作を期待するため、
//       `Send`でなく`Sync`でない生ポインタをマーク。
pub struct Window(PhantomData<*const ()>);

impl Window {
    // エラーダイアログを表示する関数
    //
    // HACK: ウィンドウとは関係がないので本来は関数にすべきだが、
    //       本モジュールのトップレベルで定義するとFFI宣言と名前衝突を起こし、
    //       FFI宣言を別モジュールに切り出すのも億劫なので、
    //       連関関数として定義している。
    pub fn show_error_dialog(message: &str) {
        let message = message
            .encode_utf16()
            .chain(iter::once(0))
            .collect::<Vec<_>>();
        unsafe { show_error_dialog(message.as_ptr()) };
    }
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        let title = title
            .encode_utf16()
            .chain(iter::once(0))
            .collect::<Vec<_>>();
        let result = unsafe { create_window(title.as_ptr(), width, height) };
        if result == 0 {
            panic_log("failed to create a window");
        }
        Self(PhantomData)
    }

    pub fn destroy(self) {
        unsafe { destroy_window() };
    }
}

impl Window {
    pub fn process_events(&self) -> bool {
        unsafe { process_window_events() != 0 }
    }

    pub fn get_current_client_size(&self) -> Option<vk::Extent2D> {
        let size = unsafe { get_current_client_size() };
        if size.width == 0 || size.height == 0 {
            None
        } else {
            Some(vk::Extent2D {
                width: size.width,
                height: size.height,
            })
        }
    }

    pub fn create_surface(&self, win32_surface_loader: &Instance) -> VkResult<vk::SurfaceKHR> {
        unsafe {
            let instance = get_instance_handle();
            if instance.is_null() {
                return Err(vk::Result::ERROR_DEVICE_LOST);
            }

            let window = get_window_handle();
            if window.is_null() {
                return Err(vk::Result::ERROR_DEVICE_LOST);
            }

            let ci = vk::Win32SurfaceCreateInfoKHR::default()
                .hinstance(instance as isize)
                .hwnd(window as isize);
            win32_surface_loader.create_win32_surface(&ci, None)
        }
    }

    pub fn toggle_fullscreen(&self) {
        unsafe { toggle_fullscreen() };
    }

    pub fn get_input_state(&self, code: u32) -> bool {
        unsafe { get_input_state(code) != 0 }
    }

    pub fn get_scale_factor(&self) -> f32 {
        1.0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct WindowSize {
    width: u32,
    height: u32,
}

#[link(name = "window", kind = "static")]
unsafe extern "C" {
    fn show_error_dialog(message: *const u16);
    fn get_instance_handle() -> *mut c_void;
    fn get_window_handle() -> *mut c_void;
    fn create_window(title: *const u16, width: u32, height: u32) -> u8;
    fn destroy_window();
    fn process_window_events() -> u8;
    fn get_current_client_size() -> WindowSize;
    fn toggle_fullscreen();
    fn get_input_state(code: u32) -> u8;
}
