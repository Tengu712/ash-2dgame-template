//! アプリケーションの定数を羅列するモジュール

use ash::vk;
use std::ffi::CStr;

pub const WINDOW_TITLE: &str = "ash-2dgame-template";
pub const SCREEN_WIDTH: u32 = 1280;
pub const SCREEN_HEIGHT: u32 = 720;

pub const APPLICATION_NAME: &CStr = c"ash-2dgame-template";
pub const APPLICATION_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);
