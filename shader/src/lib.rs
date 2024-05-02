#![cfg_attr(
    target_arch = "spirv",
    no_std,
)]

use spirv_std::{spirv, RuntimeArray};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use glam::*;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct ShaderConstants {
    pub pixel_width: u32,
    pub pixel_height: u32,

    pub screen_width: f32,
    pub screen_height: f32,
    pub screen_depth: f32,
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct InstancedQuad {
    pub top_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

#[spirv(fragment)]
pub fn fragment(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] quads: &[InstancedQuad],
    output: &mut Vec4
) {
    *output = quads[0].color;
}

#[spirv(vertex)]
pub fn vertex(
    // #[spirv(instance_index)] instance_index: i32,
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let x = ((vert_id << 1) & 2) as f32;
    let y = (vert_id & 2) as f32;
    let uv = vec2(x, y);

    let pos = 2.0 * uv - Vec2::ONE;
    *out_pos = pos.extend(0.0).extend(1.0);
}
