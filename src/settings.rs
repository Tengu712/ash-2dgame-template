//! アプリケーションの定数を羅列するモジュール

use ash::vk;
use std::ffi::CStr;

pub const APPLICATION_NAME: &CStr = c"ash-2dgame-template";
pub const APPLICATION_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);
