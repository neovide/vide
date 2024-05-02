#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use spirv_std::glam::*;

use crate::shape::*;
use crate::utils::*;

#[derive(Copy, Clone)]
pub struct Sphere {
    radius: f32
}

pub fn sphere(radius: f32) -> ShapeWrapper<Sphere> {
    ShapeWrapper(Sphere {
        radius
    })
}

impl Shape for Sphere {
    fn distance(self, position: Vec3) -> f32 {
        position.length() - self.radius
    }
}

#[derive(Copy, Clone)]
pub struct Plane {
    normal: Vec3,
}

pub fn plane(normal: Vec3) -> ShapeWrapper<Plane> {
    ShapeWrapper(Plane {
        normal
    })
}

impl Shape for Plane {
    fn distance(self, position: Vec3) -> f32 {
        position.dot(self.normal)
    }
}

#[derive(Copy, Clone)]
pub struct Cube {
    dimensions: Vec3,
}

pub fn cube(dimensions: Vec3) -> ShapeWrapper<Cube> {
    ShapeWrapper(Cube {
        dimensions
    })
}

impl Shape for Cube {
    fn distance(self, position: Vec3) -> f32 {
        let q = abs(position) - self.dimensions;
        q.max(Vec3::ZERO).length() + q.x.max(q.y).max(q.z).min(0.0)
    }
}

#[derive(Copy, Clone)]
pub struct Cylinder {
    height: f32,
    radius: f32,
}

pub fn cylinder(height: f32, radius: f32) -> ShapeWrapper<Cylinder> {
    ShapeWrapper(Cylinder {
        height,
        radius,
    })
}

impl Shape for Cylinder {
    fn distance(self, position: Vec3) -> f32 {
        let d = abs2d(vec2(position.xz().length(), position.y)) - vec2(self.radius, self.height);
        d.x.max(d.y).min(0.0) + d.max(Vec2::ZERO).length()
    }
}
