//! リソースファイルを列挙するモジュール
//!
//! NOTE: バイナリサイズが気になったり、暗号化したかったりしたら、
//!       この方式だとまずい。

use glam::Vec4;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy)]
pub struct Resource(pub &'static [u8]);

impl Hash for Resource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0.as_ptr() as usize).hash(state);
    }
}

impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0.as_ptr(), other.0.as_ptr())
    }
}

impl Eq for Resource {}

pub static IMAGE: Resource = Resource(include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/res/image.png"
)));

pub const UV_WHITE: Vec4 = Vec4::new(0.99, 0.99, 0.01, 0.01);
pub const UV_CIRCLE: Vec4 = Vec4::new(0.0, 0.5, 0.125, 0.5);
pub const UV_TITLE: Vec4 = Vec4::new(0.0, 0.0, 1.0, 0.5);
