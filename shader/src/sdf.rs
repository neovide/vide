use spirv_std::glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::ShaderConstants;
use crate::shape::*;
use crate::primitives::*;
use crate::model::model;

use spirv_std::glam::swizzles::Vec2Swizzles;

const MIN_DISTANCE: f32 = 0.001;
const MAX_DISTANCE: f32 = 50.0;

pub fn scene(position: Vec3) -> f32 {
    let world_dist = plane(Vec3::Y).distance(position);
    let model_dist = model().distance((position - Vec3::Y * 3.0) * 10.0) / 10.0;

    world_dist.min(model_dist)
}


const EPSILON: f32 = 0.001;
pub fn normal(position: Vec3) -> Vec3 {
    let offset = vec2(1.0, -1.0);
    (
        offset.xyy() * scene(position + EPSILON * offset.xyy()) +
        offset.yyx() * scene(position + EPSILON * offset.yyx()) +
        offset.yxy() * scene(position + EPSILON * offset.yxy()) +
        offset.xxx() * scene(position + EPSILON * offset.xxx())
    ).normalize()
}

pub fn march(original_position: Vec3, direction: Vec3) -> Vec3 {
    let mut traveled_distance = 0.0;
    while traveled_distance < MAX_DISTANCE {
        let distance = scene(original_position + direction * traveled_distance);

        if distance < MIN_DISTANCE {
            break;
        }

        traveled_distance += distance;
    }

    return original_position + direction * traveled_distance;
}

pub fn soft_shadow(original_position: Vec3, direction: Vec3, fuzz_factor: f32) -> f32 {
    let mut result: f32 = 1.0;
    let mut previous_distance = MIN_DISTANCE;
    let mut traveled_distance = MIN_DISTANCE;
    while traveled_distance < MAX_DISTANCE {
        let distance = scene(original_position + direction * traveled_distance);
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
    let intersection = march(start, direction);

    let sun: Vec3 = constants.sun.into();

    let ground_color = Vec3::splat(0.6);
    let shadow_drop_off = 2.0;

    let normal = normal(intersection);
    
    let shadow_mix = soft_shadow(intersection + normal * 0.01, sun, shadow_drop_off);

    let base_color = Vec3::ZERO.lerp(ground_color, shadow_mix.min(1.0));

    apply_fog(base_color, (intersection - start).length())
}
