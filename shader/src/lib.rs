#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

mod shape;
mod primitives;
mod utils;

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::glam::*;

use shape::*;
use primitives::*;
use utils::*;

const MAX_ITERATIONS: usize = 100;
const MIN_DISTANCE: f32 = 0.001;

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

    pub time: f32,
}

pub fn bend(position: Vec3, amount: f32) -> Vec3 {
    let cos = (amount * position.x).cos() * (amount * position.y).sin();
    let sin = (amount * position.x).sin() * (amount * position.y).cos();
    let matrix = Mat2::from_cols_array(&[cos, -sin, sin, cos]);
    (matrix * position.truncate()).extend(position.z)
}

pub fn terrain(mut position: Vec3) -> f32 {
    position += vec3(46380.0, 71833.0, 0.0);
    fbm(vec2(position.x / 100.0, position.z / 100.0)) * 50.0
}

pub fn sdf(position: Vec3, _time: f32) -> f32 {
    let position = bend(position, 0.05);
    let ground = plane(Vec3::ZERO, Vec3::Y);
    let (ground_distance, _) = ground.distance(position);
    let displacement = terrain(position);

    ground_distance + displacement
}

pub fn march(mut position: Vec3, direction: Vec3, time: f32) -> Vec3 {
    for _ in 0..MAX_ITERATIONS {
        let distance = sdf(position, time);
        if distance < MIN_DISTANCE {
            break;
        }

        position += direction * distance;
    }

    return position;
}

#[spirv(fragment)]
pub fn fragment(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4
) {
    let position: Vec3 = constants.position.into();
    let forward: Vec3 = constants.forward.into();
    let right = forward.cross(Vec3::Y).normalize();
    let up = forward.cross(-right).normalize();

    let uv = vec2(frag_coord.x, frag_coord.y);
    let mut uv = (uv - 0.5 * vec2(constants.pixel_width as f32, constants.pixel_height as f32))
        / constants.pixel_height as f32;
    uv.y = -uv.y;

    let screen_center = position + forward * constants.screen_depth;

    let target_pixel = screen_center + 
        up * uv.y * constants.screen_height +
        right * uv.x * constants.screen_width;

    let direction = (target_pixel - position).normalize();
    let location = march(position, direction, constants.time);

    let distance = (position - location).length();

    let color = Vec3::splat(distance / 200.0);

    *output = color.extend(1.0);
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
