use glam::*;
use spirv_std::{image::Image2d, spirv, Sampler};

use crate::ShaderConstants;

#[derive(Copy, Clone, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct InstancedGlyph {
    pub bottom_left: Vec2,
    pub atlas_top_left: Vec2,
    pub atlas_size: Vec2,
    // Need a Vec2 of padding here so that the first 4 fields
    // Are some multiple of 16 bytes in size.
    // Vec2s are 8 bytes, Vec4s are 16 bytes.
    pub _padding: Vec2,
    pub color: Vec4,
}

#[spirv(vertex)]
pub fn glyph_vertex(
    #[spirv(instance_index)] instance_index: i32,
    #[spirv(vertex_index)] vert_index: i32,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] glyphs: &[InstancedGlyph],
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

    let instance = glyphs[instance_index as usize];
    let vertex_pixel_pos =
        instance.bottom_left + (unit_vertex_pos - vec2(0., 1.)) * instance.atlas_size;

    let final_position =
        vec2(0.0, 2.0) + vertex_pixel_pos / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0;
    *out_position = final_position.extend(0.0).extend(1.0);

    *out_atlas_position = instance.atlas_top_left / constants.atlas_size
        + unit_vertex_pos * instance.atlas_size / constants.atlas_size;
}

#[spirv(fragment)]
pub fn glyph_fragment(
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] glyphs: &[InstancedGlyph],
    #[spirv(descriptor_set = 0, binding = 1)] atlas: &Image2d,
    #[spirv(descriptor_set = 1, binding = 0)] surface: &Image2d,
    #[spirv(descriptor_set = 1, binding = 1)] sampler: &Sampler,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(flat)] instance_index: i32,
    #[spirv(frag_coord)] surface_position: Vec4,
    atlas_position: Vec2,
    out_color: &mut Vec4,
) {
    let glyph = glyphs[instance_index as usize];
    // Here we have to sample specifically the 0 LOD. I don't
    // fully understand why, but I think it has to do with how
    // the spirv is generated.
    // More details here: https://github.com/gfx-rs/wgpu-rs/issues/912
    let surface_color =
        surface.sample_by_lod(*sampler, surface_position.xy() / constants.surface_size, 0.);
    let mask_color = atlas.sample_by_lod(*sampler, atlas_position, 0.);
    *out_color = glyph.color * mask_color + (1.0 - glyph.color.w * mask_color) * surface_color;
}
