use glam::*;
use spirv_std::spirv;

use crate::ShaderConstants;

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct InstancedQuad {
    pub top_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

#[spirv(vertex)]
pub fn vertex(
    #[spirv(instance_index)] instance_index: i32,
    #[spirv(vertex_index)] vert_index: i32,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] quads: &[InstancedQuad],
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(position, invariant)] out_position: &mut Vec4,
    out_instance_index: &mut i32,
) {
    *out_instance_index = instance_index;

    let unit_vertex_pos = match vert_index {
        0 => vec2(0.0, 0.0),
        1 => vec2(1.0, 0.0),
        2 => vec2(1.0, 1.0),
        3 => vec2(0.0, 0.0),
        4 => vec2(1.0, 1.0),
        5 => vec2(0.0, 1.0),
        _ => unreachable!(),
    };

    let instance = quads[instance_index as usize];
    let vertex_pixel_pos = instance.top_left + unit_vertex_pos * instance.size;

    let final_position =
        vec2(0.0, 2.0) + vertex_pixel_pos / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0;
    *out_position = final_position.extend(0.0).extend(1.0);
}

#[spirv(fragment)]
pub fn fragment(
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] quads: &[InstancedQuad],
    #[spirv(flat)] instance_index: i32,
    out_color: &mut Vec4,
) {
    let quad = quads[instance_index as usize];
    *out_color = quad.color;
}
