#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use spirv_std::glam::*;

use crate::shape_2d::*;
use crate::utils::*;

#[derive(Copy, Clone)]
pub struct Circle {
    radius: f32
}

pub fn circle(radius: f32) -> Shape2DWrapper<Circle> {
    Shape2DWrapper(Circle {
        radius
    })
}

impl Shape2D for Circle {
    fn distance(self, position: Vec2) -> f32 {
        position.length() - self.radius
    }
}

#[derive(Copy, Clone)]
pub struct Plane {
    normal: Vec2,
}

pub fn plane(normal: Vec2) -> Shape2DWrapper<Plane> {
    Shape2DWrapper(Plane {
        normal
    })
}

impl Shape2D for Plane {
    fn distance(self, position: Vec2) -> f32 {
        position.dot(self.normal)
    }
}

#[derive(Copy, Clone)]
pub struct Rectangle {
    dimensions: Vec2,
}

pub fn rectangle(dimensions: Vec2) -> Shape2DWrapper<Rectangle> {
    Shape2DWrapper(Rectangle {
        dimensions
    })
}

impl Shape2D for Rectangle {
    fn distance(self, position: Vec2) -> f32 {
        let q = abs_2d(position) - self.dimensions;
        q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0)
    }
}
