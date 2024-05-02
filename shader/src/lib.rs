#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

mod primitives_3d;
pub mod shape_3d;
mod primitives_2d;
pub mod shape_2d;
mod utils;
pub mod model;
pub mod sdf;

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::glam::*;

use sdf::compute_color;
use model::ModelConstants;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ShaderConstants {
    pub pixel_width: u32,
    pub pixel_height: u32,

    pub screen_width: f32,
    pub screen_height: f32,
    pub screen_depth: f32,

    pub position: [f32; 3],
    pub forward: [f32; 3],

    pub sun: [f32; 3],

    pub model_constants: ModelConstants,
}

#[spirv(fragment)]
pub fn fragment(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4
) {
    let start: Vec3 = constants.position.into();
    let forward: Vec3 = constants.forward.into();
    let right = forward.cross(Vec3::Y).normalize();
    let up = right.cross(forward).normalize();

    let uv = vec2(frag_coord.x, frag_coord.y);
    let mut uv = (uv - 0.5 * vec2(constants.pixel_width as f32, constants.pixel_height as f32))
        / constants.pixel_height as f32;
    uv.y = -uv.y;

    let screen_center = start + forward * constants.screen_depth;

    let target_pixel = screen_center + 
        up * uv.y * constants.screen_height +
        right * uv.x * constants.screen_width;

    let direction = (target_pixel - start).normalize();

    *output = compute_color(start, direction, constants).extend(1.0);
}

#[spirv(vertex)]
pub fn vertex(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let x = ((vert_id << 1) & 2) as f32;
    let y = (vert_id & 2) as f32;
    let uv = vec2(x, y);

    let pos = 2.0 * uv - Vec2::ONE;
    *out_pos = pos.extend(0.0).extend(1.0);
}
