#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use spirv_std::glam::Vec3;

use crate::shape::*;

#[derive(Copy, Clone)]
pub struct Sphere {
    color: Vec3,
    center: Vec3,
    radius: f32
}

pub fn sphere(center: Vec3, radius: f32) -> ShapeWrapper<Sphere> {
    ShapeWrapper(Sphere {
        color: Vec3::ONE,
        center, 
        radius
    })
}

impl Shape for Sphere {
    fn distance(self, position: Vec3) -> (f32, Vec3) {
        (position.distance(self.center) - self.radius, self.color)
    }
}

#[derive(Copy, Clone)]
pub struct Plane {
    color: Vec3,
    point: Vec3,
    normal: Vec3,
}

pub fn plane(point: Vec3, normal: Vec3) -> ShapeWrapper<Plane> {
    ShapeWrapper(Plane {
        color: Vec3::ONE,
        point,
        normal
    })
}

impl Shape for Plane {
    fn distance(self, position: Vec3) -> (f32, Vec3) {
        let relative_position = position - self.point;

        (relative_position.dot(self.normal), self.color)
    }
}
