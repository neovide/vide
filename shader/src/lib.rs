#![cfg_attr(target_arch = "spirv", no_std)]

mod glyph;
mod quad;

use glam::Vec2;
pub use glyph::*;
pub use quad::*;

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct ShaderConstants {
    pub surface_size: Vec2,
    pub atlas_size: Vec2,
}
