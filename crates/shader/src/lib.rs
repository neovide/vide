#![cfg_attr(target_arch = "spirv", no_std)]

mod glyph;
mod path;
mod quad;
mod sprite;

pub use glyph::*;
pub use path::*;
pub use quad::*;
pub use sprite::*;

#[cfg(target_arch = "spirv")]
use spirv_std::glam::*;

#[cfg(not(target_arch = "spirv"))]
use glam::*;

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct ShaderConstants {
    pub surface_size: Vec2,
    pub atlas_size: Vec2,
    pub clip: Vec4,
}
