use std::{ffi::c_void, iter};

#[link(name = "window", kind = "static")]
unsafe extern "C" {
    fn create_window(title: *const u16, width: u32, height: u32) -> *mut c_void;
    fn destroy_window(window: *mut c_void);
    fn process_window_events() -> u8;
}

pub struct Window {
    window: *mut c_void,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        let title = title
            .encode_utf16()
            .chain(iter::once(0))
            .collect::<Vec<_>>();
        let window = unsafe { create_window(title.as_ptr(), width, height) };
        if window.is_null() {
            panic!("failed to create a window");
        }
        Self { window }
    }

    pub fn process_events(&self) -> bool {
        unsafe { process_window_events() != 0 }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { destroy_window(self.window) };
    }
}
