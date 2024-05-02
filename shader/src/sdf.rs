use spirv_std::glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::ShaderConstants;
use crate::shape::*;
use crate::primitives::*;
use crate::utils::*;

use spirv_std::glam::swizzles::Vec2Swizzles;

const MIN_DISTANCE: f32 = 0.001;
const MAX_DISTANCE: f32 = 100.0;

pub fn scene(position: Vec3, time: f32) -> f32 {
    let ball_x = time.sin() * 2.0;
    let ball_y = time.cos() * 4.0;

    let ball = sphere(1.0) + Vec3::Z * 5.0 + vec3(ball_x, ball_y, 0.0);
    let ground = plane(Vec3::Y);

    let arch = 
        cube(vec3(3.0, 6.0, 1.0)) - // Base
        (cylinder(6.0, 2.0) * Quat::from_rotation_x(core::f32::consts::FRAC_PI_2) + (Vec3::Y * 3.0)) - // Arch
        cube(vec3(2.0, 3.0, 2.0)); // Walkway

    let arch = arch + (Vec3::Z * 7.0);

    let field = ground.smooth_union(ball, 1.0).smooth_union(arch, 0.8);

    field.distance(position)
}

const EPSILON: f32 = 0.001;
pub fn normal(position: Vec3, time: f32) -> Vec3 {
    let offset = vec2(1.0, -1.0);
    (
        offset.xyy() * scene(position + EPSILON * offset.xyy(), time) +
        offset.yyx() * scene(position + EPSILON * offset.yyx(), time) +
        offset.yxy() * scene(position + EPSILON * offset.yxy(), time) +
        offset.xxx() * scene(position + EPSILON * offset.xxx(), time)
    ).normalize()
}

pub fn march(original_position: Vec3, direction: Vec3, time: f32) -> Vec3 {
    let mut traveled_distance = 0.0;
    while traveled_distance < MAX_DISTANCE {
        let distance = scene(original_position + direction * traveled_distance, time);

        if distance < MIN_DISTANCE {
            break;
        }

        traveled_distance += distance;
    }

    return original_position + direction * traveled_distance;
}

pub fn soft_shadow(original_position: Vec3, direction: Vec3, time: f32, fuzz_factor: f32) -> f32 {
    let mut result: f32 = 1.0;
    let mut previous_distance = MIN_DISTANCE;
    let mut traveled_distance = MIN_DISTANCE;
    while traveled_distance < MAX_DISTANCE {
        let distance = scene(original_position + direction * traveled_distance, time);
        if distance < MIN_DISTANCE {
            return 0.0;
        }
        let y = distance * distance / (2.0 * previous_distance);
        let d = (distance * distance - y * y).sqrt();
        result = result.min(fuzz_factor * d / (traveled_distance - y).max(0.0));
        previous_distance = distance;
        traveled_distance += distance;
    }

    result
}

pub fn apply_fog(base_color: Vec3, distance_traveled: f32) -> Vec3 {
    let fog_color = Vec3::ONE;

    base_color.lerp(fog_color, distance_traveled.min(MAX_DISTANCE) / MAX_DISTANCE)
}

pub fn compute_color(start: Vec3, direction: Vec3, constants: &ShaderConstants) -> Vec3 {
    let intersection = march(start, direction, constants.time);

    let sun: Vec3 = constants.sun.into();

    let ground_color = Vec3::splat(0.6);
    let shadow_drop_off = 2.0;

    let normal = normal(intersection, constants.time);
    
    let shadow_mix = soft_shadow(intersection + normal * 0.01, sun, constants.time, shadow_drop_off);

    let base_color = Vec3::ZERO.lerp(ground_color, shadow_mix.min(1.0));

    apply_fog(base_color, (intersection - start).length())
}
