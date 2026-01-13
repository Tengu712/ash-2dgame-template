use crate::logs::*;
use ash::{khr::xcb_surface::Instance, prelude::VkResult, vk};
use std::{
    ffi::{CString, c_char, c_void},
    marker::PhantomData,
};

/// Xcb製ウィンドウ
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
        let message = CString::new(message);
        let message = message.expect_log("failed to convert an error message to the OS string");
        unsafe { show_error_dialog(message.as_ptr()) };
    }
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        let title = CString::new(title);
        let title = title.expect_log("failed to convert the title string to the OS string");
        let result = unsafe { create_window(title.as_ptr(), width, height) };
        if result == 0 {
            panic_log("failed to create a window");
        }
        Self(PhantomData)
    }

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

    pub fn create_surface(&self, xcb_surface_loader: &Instance) -> VkResult<vk::SurfaceKHR> {
        unsafe {
            let connection_and_window = get_connection_and_window();
            if connection_and_window.connection.is_null() || connection_and_window.window == 0 {
                return Err(vk::Result::ERROR_DEVICE_LOST);
            }

            let ci = vk::XcbSurfaceCreateInfoKHR::default()
                .connection(connection_and_window.connection)
                .window(connection_and_window.window);
            xcb_surface_loader.create_xcb_surface(&ci, None)
        }
    }

    pub fn toggle_fullscreen(&self) {
        unsafe { toggle_fullscreen() }
    }

    pub fn get_input_state(&self, code: u32) -> bool {
        unsafe { get_input_state(code) != 0 }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { destroy_window() };
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct WindowSize {
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ConnectionAndWindow {
    connection: *mut c_void,
    window: u32,
}

#[link(name = "window", kind = "static")]
unsafe extern "C" {
    fn show_error_dialog(message: *const c_char);
    fn get_connection_and_window() -> ConnectionAndWindow;
    fn create_window(title: *const c_char, width: u32, height: u32) -> u8;
    fn destroy_window();
    fn process_window_events() -> u8;
    fn get_current_client_size() -> WindowSize;
    fn toggle_fullscreen();
    fn get_input_state(code: u32) -> u8;
}
