#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use spirv_std::glam::*;

use crate::shape_2d::Shape2D;
use crate::shape_3d::*;
use crate::utils::*;

#[derive(Copy, Clone)]
pub struct Sphere {
    radius: f32
}

pub fn sphere(radius: f32) -> Shape3DWrapper<Sphere> {
    Shape3DWrapper(Sphere {
        radius
    })
}

impl Shape3D for Sphere {
    fn distance(self, position: Vec3) -> f32 {
        position.length() - self.radius
    }
}

#[derive(Copy, Clone)]
pub struct Plane {
    normal: Vec3,
}

pub fn plane(normal: Vec3) -> Shape3DWrapper<Plane> {
    Shape3DWrapper(Plane {
        normal
    })
}

impl Shape3D for Plane {
    fn distance(self, position: Vec3) -> f32 {
        position.dot(self.normal)
    }
}

#[derive(Copy, Clone)]
pub struct Cube {
    dimensions: Vec3,
}

pub fn cube(dimensions: Vec3) -> Shape3DWrapper<Cube> {
    Shape3DWrapper(Cube {
        dimensions
    })
}

impl Shape3D for Cube {
    fn distance(self, position: Vec3) -> f32 {
        let q = abs_3d(position) - self.dimensions;
        q.max(Vec3::ZERO).length() + q.x.max(q.y).max(q.z).min(0.0)
    }
}

#[derive(Copy, Clone)]
pub struct Cylinder {
    height: f32,
    radius: f32,
}

pub fn cylinder(height: f32, radius: f32) -> Shape3DWrapper<Cylinder> {
    Shape3DWrapper(Cylinder {
        height,
        radius,
    })
}

impl Shape3D for Cylinder {
    fn distance(self, position: Vec3) -> f32 {
        let d = abs_2d(vec2(position.xz().length(), position.y)) - vec2(self.radius, self.height);
        d.x.max(d.y).min(0.0) + d.max(Vec2::ZERO).length()
    }
}
