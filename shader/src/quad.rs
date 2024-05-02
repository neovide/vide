use glam::*;
use spirv_std::{image::Image2d, spirv, Sampler};

use crate::ShaderConstants;

const GUASSIAN_WEIGHT_FACTOR: f32 = 1. / 1003.;
const GUASSIAN_WEIGHTS: [[f32; 4]; 4] = [
    [0., 0., 1., 2.],
    [0., 3., 13., 22.],
    [1., 13., 59., 97.],
    [2., 22., 97., 159.],
];
const GUASSIAN_RADIUS: i32 = 3;

#[derive(Copy, Clone)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(bytemuck::Pod, bytemuck::Zeroable, Default, Debug)
)]
#[repr(C)]
pub struct InstancedQuad {
    pub top_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub blur: u32, // 0 == no blur, non 0 = blur with 5x5 samples
    pub _padding: Vec3,
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
    #[spirv(descriptor_set = 1, binding = 0)] surface: &Image2d,
    #[spirv(descriptor_set = 1, binding = 1)] sampler: &Sampler,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(flat)] instance_index: i32,
    #[spirv(frag_coord)] surface_position: Vec4,
    out_color: &mut Vec4,
) {
    let quad = quads[instance_index as usize];

    if quad.blur != 0 {
        // Blur the quad background by sampling surrounding pixels
        // and averaging them using the Gaussian blur kernel.
        // The weights are defined by the top left quadrant of the kernel
        // and then sampled using the symmetry of the kernel.
        *out_color = Vec4::ZERO;
        for y in -GUASSIAN_RADIUS..=GUASSIAN_RADIUS {
            for x in -GUASSIAN_RADIUS..=GUASSIAN_RADIUS {
                let weight = GUASSIAN_WEIGHT_FACTOR
                    * GUASSIAN_WEIGHTS[(GUASSIAN_RADIUS - x.abs()) as usize]
                        [(GUASSIAN_RADIUS - y.abs()) as usize];
                let offset = vec2(x as f32, y as f32);
                let sample_pos = (surface_position.xy() + offset) / constants.surface_size;
                let sample = surface.sample_by_lod(*sampler, sample_pos, 0.);
                *out_color += sample * weight;
            }
        }

        *out_color = *out_color * quad.color;
    } else {
        *out_color = quad.color;
    }
}
