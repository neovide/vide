#![cfg_attr(
    target_arch = "spirv",
    no_std,
)]

mod quad;

pub use quad::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ShaderConstants {
    pub pixel_width: u32,
    pub pixel_height: u32,
}
