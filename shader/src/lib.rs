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

    pub sun: [f32; 3],

    pub time: f32,
}

pub fn sdf(position: Vec3, time: f32) -> f32 {
    let ball_x = time.sin() * 2.0;
    let ball_y = time.cos() * 4.0;

    let ball = sphere(Vec3::Z * 5.0 + vec3(ball_x, ball_y, 0.0), 1.0);
    let ground = plane(Vec3::ZERO, Vec3::Y);

    let arch = cube(vec3(3.0, 6.0, 1.0)) - cylinder(vec3(0.0, 3.0, 3.0), vec3(0.0, 3.0, -3.0), 2.0) - cube(vec3(2.0, 3.0, 2.0));
    let arch = arch + (Vec3::Z * 7.0);

    let field = ground.smooth_union(ball, 1.0).smooth_union(arch, 0.8);

    let (distance, _) = field.distance(position);

    distance
}

const EPSILON: f32 = 0.001;
pub fn normal(position: Vec3, time: f32) -> Vec3 {
    vec3(
        sdf(position + Vec3::X * EPSILON, time) - sdf(position - Vec3::X * EPSILON, time),
        sdf(position + Vec3::Y * EPSILON, time) - sdf(position - Vec3::Y * EPSILON, time),
        sdf(position + Vec3::Z * EPSILON, time) - sdf(position - Vec3::Z * EPSILON, time)).normalize()
}

pub fn march(mut position: Vec3, direction: Vec3, time: f32) -> (Vec3, f32) {
    let mut closest = core::f32::MAX;
    for i in 0..MAX_ITERATIONS {
        let distance = sdf(position, time);
        position += direction * distance;

        closest = closest.min(2.0 * distance / (i as f32));
        if distance < MIN_DISTANCE {
            return (position, closest);
        }
    }

    return (position, closest);
}

pub fn apply_fog(base_color: Vec3, distance_traveled: f32) -> Vec3 {
    let fog_color = Vec3::ONE;
    let fog_max = 100.0;

    base_color.lerp(fog_color, distance_traveled.min(fog_max) / fog_max)
}

pub fn compute_color(start: Vec3, direction: Vec3, constants: &ShaderConstants) -> Vec3 {
    let (intersection, _) = march(start, direction, constants.time);

    let sun: Vec3 = constants.sun.into();

    let ground_color = Vec3::splat(0.9);
    let shadow_color = Vec3::splat(0.0125);
    let shadow_drop_off = 3.0;

    let normal = normal(intersection, constants.time);
    // let sun_angle = normal.dot(sun);
    
    let (_, closest_obscurer) = march(intersection + normal * 0.1, sun, constants.time);
    let shadow_mix_factor = closest_obscurer.min(shadow_drop_off) / shadow_drop_off;

    let base_color = shadow_color.lerp(ground_color, closest_obscurer.min(1.0));

    apply_fog(base_color, (intersection - start).length())
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
    let up = forward.cross(-right).normalize();

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
