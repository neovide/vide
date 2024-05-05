#![cfg_attr(target_arch = "spirv", no_std)]

mod glyph;
mod path;
mod quad;
mod sprite;

pub use glyph::*;
pub use path::*;
pub use quad::*;
pub use sprite::*;

use glam::*;
use rust_embed::RustEmbed;
use wgpu::{Device, ShaderModule, ShaderModuleDescriptor, ShaderSource};

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct ShaderConstants {
    pub surface_size: Vec2,
    pub atlas_size: Vec2,
    pub clip: Vec4,
}

#[derive(RustEmbed)]
#[folder = "wgsl/"]
struct Shader;

pub fn load_shader(device: &Device) -> ShaderModule {
    let mut source = String::new();
    for path in Shader::iter() {
        if let Some(file) = Shader::get(&path) {
            source += std::str::from_utf8(file.data.as_ref()).unwrap();
        }
    }

    device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shader"),
        source: ShaderSource::Wgsl(source.into()),
    })
}
