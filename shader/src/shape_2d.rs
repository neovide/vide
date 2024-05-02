use core::ops::{Add, Mul, Sub, Rem};

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::glam::*;

use crate::shape_3d::{Shape3D, Shape3DWrapper};

pub trait Shape2D: Copy {
    fn distance(self, position: Vec2) -> f32;

    fn translate(self, translation: Vec2) -> Shape2DWrapper<Translation<Self>> {
        Shape2DWrapper(Translation(self, translation))
    }

    fn rotate(self, rotation: Quat) -> Shape2DWrapper<Rotation<Self>> {
        Shape2DWrapper(Rotation(self, rotation))
    }

    fn repeat(self, period: Vec2) -> Shape2DWrapper<Repeat<Self>> {
        Shape2DWrapper(Repeat(self, period))
    }

    fn scale(self, scale: f32) -> Shape2DWrapper<Scale<Self>> {
        Shape2DWrapper(Scale(self, scale))
    }

    fn expand(self, amount: f32) -> Shape2DWrapper<Expansion<Self>> {
        Shape2DWrapper(Expansion(self, amount))
    }

    fn union<S>(self, other: S) -> Shape2DWrapper<Union<Self, S>> {
        Shape2DWrapper(Union(self, other))
    }

    fn smooth_union<S>(self, other: S, amount: f32) -> Shape2DWrapper<SmoothUnion<Self, S>> {
        Shape2DWrapper(SmoothUnion(self, other, amount))
    }

    fn intersect<S>(self, other: S) -> Shape2DWrapper<Intersection<Self, S>> {
        Shape2DWrapper(Intersection(self, other))
    }

    fn difference<S>(self, other: S) -> Shape2DWrapper<Difference<Self, S>> {
        Shape2DWrapper(Difference(self, other))
    }

    fn extrude(self, amount: f32) -> Shape3DWrapper<Extrusion<Self>> {
        Shape3DWrapper(Extrusion(self, amount))
    }
}

#[derive(Copy, Clone)]
pub struct Shape2DWrapper<A>(pub A);

impl<A: Shape2D> Shape2D for Shape2DWrapper<A> {
    fn distance(self, position: Vec2) -> f32 {
        self.0.distance(position)
    }
}

#[derive(Copy, Clone)]
pub struct Translation<A>(A, Vec2);

impl<A: Shape2D> Shape2D for Translation<A> {
    fn distance(self, position: Vec2) -> f32 {
        self.0.distance(position - self.1)
    }
}

#[derive(Copy, Clone)]
pub struct Rotation<A>(A, Quat);

impl<A: Shape2D> Shape2D for Rotation<A> {
    fn distance(self, position: Vec2) -> f32 {
        self.0.distance((self.1.inverse() * position.extend(0.0)).truncate())
    }
}

#[derive(Copy, Clone)]
pub struct Scale<A>(A, f32);

impl<A: Shape2D> Shape2D for Scale<A> {
    fn distance(self, position: Vec2) -> f32 {
        self.0.distance(position / self.1) * self.1
    }
}

#[derive(Copy, Clone)]
pub struct Repeat<A>(A, Vec2);

fn modulo(x: f32, m: f32) -> f32 {
    let x = x + m / 2.0;
    (((x % m) + m) % m) - m / 2.0
}

impl<A: Shape2D> Shape2D for Repeat<A> {
    fn distance(self, position: Vec2) -> f32 {
        let period = self.1;
        let position = vec2(
            modulo(position.x, period.x), 
            modulo(position.y, period.y));

        self.0.distance(position)
    }
}

#[derive(Copy, Clone)]
pub struct Union<A, B>(A, B);

impl<A: Shape2D, B: Shape2D> Shape2D for Union<A, B> {
    fn distance(self, position: Vec2) -> f32 {
        self.0.distance(position).min(self.1.distance(position))
    }
}

#[derive(Copy, Clone)]
pub struct SmoothUnion<A, B>(A, B, f32);

impl<A: Shape2D, B: Shape2D> Shape2D for SmoothUnion<A, B> {
    fn distance(self, position: Vec2) -> f32 {
        let distance_a = self.0.distance(position);
        let distance_b = self.1.distance(position);

        let h = (0.5 - (distance_b - distance_a).abs()).max(0.0) / 0.5;
        distance_a.min(distance_b) - h * h * 0.125
    }
}

#[derive(Copy, Clone)]
pub struct Intersection<A, B>(A, B);

impl<A: Shape2D, B: Shape2D> Shape2D for Intersection<A, B> {
    fn distance(self, position: Vec2) -> f32 {
        self.0.distance(position).max(self.1.distance(position))
    }
}

#[derive(Copy, Clone)]
pub struct Difference<A, B>(A, B);

impl<A: Shape2D, B: Shape2D> Shape2D for Difference<A, B> {
    fn distance(self, position: Vec2) -> f32 {
        (-self.1.distance(position)).max(self.0.distance(position))
    }
}

#[derive(Copy, Clone)]
pub struct Expansion<A>(A, f32);

impl<A: Shape2D> Shape2D for Expansion<A> {
    fn distance(self, position: Vec2) -> f32 {
        self.0.distance(position) - self.1
    }
}

#[derive(Copy, Clone)]
pub struct Extrusion<A>(A, f32);

impl<A: Shape2D> Shape3D for Extrusion<A> {
    fn distance(self, position: Vec3) -> f32 {
        let distance = self.0.distance(position.xy());
        let w = vec2(distance, position.z.abs() - self.1);
        w.x.max(w.y).min(0.0) + vec2(w.x.max(0.0), w.y.max(0.0)).length()
    }
}

// Translate via + operator
impl<A: Shape2D> Add<Vec2> for Shape2DWrapper<A> {
    type Output = Shape2DWrapper<Translation<Shape2DWrapper<A>>>;

    fn add(self, amount: Vec2) -> Self::Output {
        self.translate(amount)
    }
}

// Rotation via * operator
impl<A: Shape2D> Mul<Quat> for Shape2DWrapper<A> {
    type Output = Shape2DWrapper<Rotation<Shape2DWrapper<A>>>;

    fn mul(self, rotation: Quat) -> Self::Output {
        self.rotate(rotation)
    }
}

// Scale via * operator
impl<A: Shape2D> Mul<f32> for Shape2DWrapper<A> {
    type Output = Shape2DWrapper<Scale<Shape2DWrapper<A>>>;

    fn mul(self, scale: f32) -> Self::Output {
        self.scale(scale)
    }
}

// Union via + operator
impl<A: Shape2D, B: Shape2D> Add<Shape2DWrapper<B>> for Shape2DWrapper<A> {
    type Output = Shape2DWrapper<Union<Shape2DWrapper<A>, Shape2DWrapper<B>>>;

    fn add(self, other: Shape2DWrapper<B>) -> Self::Output {
        self.union(other)
    }
}

// Intersection via * operator
impl<A: Shape2D, B: Shape2D> Mul<Shape2DWrapper<B>> for Shape2DWrapper<A> {
    type Output = Shape2DWrapper<Intersection<Shape2DWrapper<A>, Shape2DWrapper<B>>>;

    fn mul(self, other: Shape2DWrapper<B>) -> Self::Output {
        self.intersect(other)
    }
}

// Difference via - operator
impl<A: Shape2D, B: Shape2D> Sub<Shape2DWrapper<B>> for Shape2DWrapper<A> {
    type Output = Shape2DWrapper<Difference<Shape2DWrapper<A>, Shape2DWrapper<B>>>;

    fn sub(self, other: Shape2DWrapper<B>) -> Self::Output {
        self.difference(other)
    }
}

// Repeat via % operator
impl<A: Shape2D> Rem<Vec2> for Shape2DWrapper<A> {
    type Output = Shape2DWrapper<Repeat<Shape2DWrapper<A>>>;

    fn rem(self, period: Vec2) -> Self::Output {
        self.repeat(period)
    }
}
