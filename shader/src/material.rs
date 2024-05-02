#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use spirv_std::glam::{vec3, Vec3, Quat};

use crate::utils::*;

pub trait Material: Copy {
    fn sample(self, position: Vec3) -> Vec3
}

#[derive(Copy, Clone)]
pub struct Solid {
    color: Vec3
}

impl Material for Solid {
    fn sample(self, _: Vec3) -> Vec3 {
        self.color
    }
}

#[derive(Copy, Clone)]
pub struct Checkers {
    color_a: Vec3,
    color_b: Vec3,
}

impl Material for Checkers {
    fn sample(self, position: Vec3) -> Vec3 {
        let position = position.floor();
        
        let t = (position.x + position.y + position.z) % 2.0;
        mix3d(self.color_a, self.color_b, t)
    }
}
