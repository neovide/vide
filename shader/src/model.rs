use spirv_std::glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::shape::*;
use crate::primitives::*;

pub fn model() -> impl Shape {
    let arch = 
        cube(vec3(3.0, 6.0, 1.0)) - // Base
        (cylinder(6.0, 2.0) * Quat::from_rotation_x(core::f32::consts::FRAC_PI_2) + (Vec3::Y * 3.0)) - // Arch
        cube(vec3(2.0, 3.0, 2.0)); // Walkway
    (arch + (Vec3::Z * 7.0)) * Quat::from_rotation_x(core::f32::consts::FRAC_PI_2)
}
