use spirv_std::{glam::*, spirv};

use crate::ShaderConstants;

#[derive(Copy, Clone)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Default)
)]
#[repr(C)]
// NOTE: Keep the ATTRIBS array in sync with this struct
pub struct PathVertex {
    pub color: Vec4,
    pub position: Vec2,
    pub _padding: Vec2,
}

#[spirv(vertex)]
pub fn path_vertex(
    #[spirv(push_constant)] constants: &ShaderConstants,
    color: Vec4,
    position: Vec2,
    out_color: &mut Vec4,
    #[spirv(position, invariant)] out_position: &mut Vec4,
) {
    *out_color = color;
    *out_position = (vec2(0., 2.) + position / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0)
        .extend(0.)
        .extend(1.);
}

#[spirv(fragment)]
pub fn path_fragment(color: Vec4, out_color: &mut Vec4) {
    *out_color = color * color;
}
