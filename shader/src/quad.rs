use spirv_std::spirv;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use glam::*;

use crate::ShaderConstants;

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct InstancedQuad {
    pub top_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

#[spirv(fragment)]
pub fn fragment(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] _constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] quads: &[InstancedQuad],
    #[spirv(flat)] instance_index: i32,
    output: &mut Vec4
) {
    *output = quads[instance_index as usize].color;
}

#[spirv(vertex)]
pub fn vertex(
    #[spirv(instance_index)] instance_index: i32,
    #[spirv(vertex_index)] vert_index: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] quads: &[InstancedQuad],
    #[spirv(push_constant)] constants: &ShaderConstants,
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

    let final_position = vec2(0.0, 2.0) + vertex_pixel_pos / vec2(constants.pixel_width as f32, constants.pixel_height as f32 * -1.0) * 2.0 - 1.0;
    *out_pos = final_position.extend(0.0).extend(1.0);
}
