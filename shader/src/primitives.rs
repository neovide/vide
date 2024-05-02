#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use spirv_std::glam::*;

use crate::shape::*;
use crate::utils::*;

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

#[derive(Copy, Clone)]
pub struct Cube {
    color: Vec3,
    dimensions: Vec3,
}

pub fn cube(dimensions: Vec3) -> ShapeWrapper<Cube> {
    ShapeWrapper(Cube {
        color: Vec3::ONE,
        dimensions
    })
}

impl Shape for Cube {
    fn distance(self, position: Vec3) -> (f32, Vec3) {
        let q = vec3(position.x.abs(), position.y.abs(), position.z.abs()) - self.dimensions;
        let distance = 
            vec3(q.x.max(0.0), q.y.max(0.0), q.z.max(0.0)).length() + 
            q.x.max(q.y).max(q.z).min(0.0);

        (distance, self.color)
    }
}

#[derive(Copy, Clone)]
pub struct Cylinder {
    color: Vec3,
    point_a: Vec3,
    point_b: Vec3,
    radius: f32,
}

pub fn cylinder(point_a: Vec3, point_b: Vec3, radius: f32) -> ShapeWrapper<Cylinder> {
    ShapeWrapper(Cylinder {
        color: Vec3::ONE,
        point_a,
        point_b,
        radius,
    })
}

impl Shape for Cylinder {
    fn distance(self, position: Vec3) -> (f32, Vec3) {
        let ba = self.point_b - self.point_a;
        let pa = position - self.point_a;

        let baba = ba.dot(ba);
        let paba = pa.dot(ba);
        let x = (pa * baba - ba * paba).length() - self.radius * baba;
        let y = (paba - baba * 0.5).abs() - baba * 0.5;
        let x2 = x * x;
        let y2 = y * y * baba;
        let d = if x.max(y) < 0.0 { 
            -x2.min(y2) 
        } else {
            (if x > 0.0 {
                x2
            } else {
                0.0
            }) + (if y > 0.0 {
                y2
            } else {
                0.0
            })
        };

        let sign = if d >= 0.0 {
            1.0
        } else {
            -1.0
        };

        let distance = sign * d.abs().sqrt() / baba;

        (distance, self.color)
    }
}
