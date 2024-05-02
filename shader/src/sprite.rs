use glam::*;
use spirv_std::{image::Image2d, spirv, Sampler};

use crate::ShaderConstants;

#[derive(Copy, Clone, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct InstancedSprite {
    pub top_left: Vec2,
    pub size: Vec2,
    pub atlas_top_left: Vec2,
    pub atlas_size: Vec2,
    pub color: Vec4,
}

#[spirv(vertex)]
pub fn sprite_vertex(
    #[spirv(instance_index)] instance_index: i32,
    #[spirv(vertex_index)] vert_index: i32,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] sprites: &[InstancedSprite],
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(position, invariant)] out_position: &mut Vec4,
    out_instance_index: &mut i32,
    out_atlas_position: &mut Vec2,
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

    let instance = sprites[instance_index as usize];
    let vertex_pixel_pos = instance.top_left + unit_vertex_pos * instance.size;

    let final_position =
        vec2(0.0, 2.0) + vertex_pixel_pos / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0;
    *out_position = final_position.extend(0.0).extend(1.0);

    *out_atlas_position = instance.atlas_top_left / constants.atlas_size
        + unit_vertex_pos * instance.atlas_size / constants.atlas_size;
}

#[spirv(fragment)]
pub fn sprite_fragment(
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] sprites: &[InstancedSprite],
    #[spirv(descriptor_set = 0, binding = 1)] atlas: &Image2d,
    #[spirv(descriptor_set = 1, binding = 1)] sampler: &Sampler,
    #[spirv(flat)] instance_index: i32,
    atlas_position: Vec2,
    out_color: &mut Vec4,
) {
    let instance = sprites[instance_index as usize];
    // Here we have to sample specifically the 0 LOD. I don't
    // fully understand why, but I think it has to do with how
    // the spirv is generated.
    // More details here: https://github.com/gfx-rs/wgpu-rs/issues/912
    let image_color = atlas.sample_by_lod(*sampler, atlas_position, 0.);
    *out_color = instance.color * image_color;
}
