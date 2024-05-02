use core::ops::{Add, Mul, Sub, Rem};

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use spirv_std::glam::{vec3, Vec3, Quat};

pub trait Shape: Copy {
    fn distance(self, position: Vec3) -> f32;

    fn translate(self, translation: Vec3) -> ShapeWrapper<Translation<Self>> {
        ShapeWrapper(Translation(self, translation))
    }

    fn rotate(self, rotation: Quat) -> ShapeWrapper<Rotation<Self>> {
        ShapeWrapper(Rotation(self, rotation))
    }

    fn repeat(self, period: Vec3) -> ShapeWrapper<Repeat<Self>> {
        ShapeWrapper(Repeat(self, period))
    }

    fn union<S>(self, other: S) -> ShapeWrapper<Union<Self, S>> {
        ShapeWrapper(Union(self, other))
    }

    fn smooth_union<S>(self, other: S, amount: f32) -> ShapeWrapper<SmoothUnion<Self, S>> {
        ShapeWrapper(SmoothUnion(self, other, amount))
    }

    fn intersect<S>(self, other: S) -> ShapeWrapper<Intersection<Self, S>> {
        ShapeWrapper(Intersection(self, other))
    }

    fn difference<S>(self, other: S) -> ShapeWrapper<Difference<Self, S>> {
        ShapeWrapper(Difference(self, other))
    }
}

#[derive(Copy, Clone)]
pub struct ShapeWrapper<A>(pub A);

impl<A: Shape> Shape for ShapeWrapper<A> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(position)
    }
}

#[derive(Copy, Clone)]
pub struct Translation<A>(A, Vec3);

impl<A: Shape> Shape for Translation<A> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(position - self.1)
    }
}

#[derive(Copy, Clone)]
pub struct Rotation<A>(A, Quat);

impl<A: Shape> Shape for Rotation<A> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(self.1.inverse() * position)
    }
}


#[derive(Copy, Clone)]
pub struct Repeat<A>(A, Vec3);

fn modulo(x: f32, m: f32) -> f32 {
    let x = x + m / 2.0;
    (((x % m) + m) % m) - m / 2.0
}

impl<A: Shape> Shape for Repeat<A> {
    fn distance(self, position: Vec3) -> f32 {
        let period = self.1;
        let position = vec3(
            modulo(position.x, period.x), 
            modulo(position.y, period.y), 
            modulo(position.z, period.z));

        self.0.distance(position)
    }
}

#[derive(Copy, Clone)]
pub struct Union<A, B>(A, B);

impl<A: Shape, B: Shape> Shape for Union<A, B> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(position).min(self.1.distance(position))
    }
}

#[derive(Copy, Clone)]
pub struct SmoothUnion<A, B>(A, B, f32);

impl<A: Shape, B: Shape> Shape for SmoothUnion<A, B> {
    fn distance(self, position: Vec3) -> f32 {
        let distance_a = self.0.distance(position);
        let distance_b = self.1.distance(position);

        let h = (0.5 - (distance_b - distance_a).abs()).max(0.0) / 0.5;
        distance_a.min(distance_b) - h * h * 0.125
    }
}

#[derive(Copy, Clone)]
pub struct Intersection<A, B>(A, B);

impl<A: Shape, B: Shape> Shape for Intersection<A, B> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(position).max(self.1.distance(position))
    }
}

#[derive(Copy, Clone)]
pub struct Difference<A, B>(A, B);

impl<A: Shape, B: Shape> Shape for Difference<A, B> {
    fn distance(self, position: Vec3) -> f32 {
        (-self.1.distance(position)).max(self.0.distance(position))
    }
}

// Translate via + operator
impl<A: Shape> Add<Vec3> for ShapeWrapper<A> {
    type Output = ShapeWrapper<Translation<ShapeWrapper<A>>>;

    fn add(self, amount: Vec3) -> Self::Output {
        self.translate(amount)
    }
}

// Rotation via * operator
impl<A: Shape> Mul<Quat> for ShapeWrapper<A> {
    type Output = ShapeWrapper<Rotation<ShapeWrapper<A>>>;

    fn mul(self, rotation: Quat) -> Self::Output {
        self.rotate(rotation)
    }
}

// Union via + operator
impl<A: Shape, B: Shape> Add<ShapeWrapper<B>> for ShapeWrapper<A> {
    type Output = ShapeWrapper<Union<ShapeWrapper<A>, ShapeWrapper<B>>>;

    fn add(self, other: ShapeWrapper<B>) -> Self::Output {
        self.union(other)
    }
}

// Intersection via * operator
impl<A: Shape, B: Shape> Mul<ShapeWrapper<B>> for ShapeWrapper<A> {
    type Output = ShapeWrapper<Intersection<ShapeWrapper<A>, ShapeWrapper<B>>>;

    fn mul(self, other: ShapeWrapper<B>) -> Self::Output {
        self.intersect(other)
    }
}

// Difference via - operator
impl<A: Shape, B: Shape> Sub<ShapeWrapper<B>> for ShapeWrapper<A> {
    type Output = ShapeWrapper<Difference<ShapeWrapper<A>, ShapeWrapper<B>>>;

    fn sub(self, other: ShapeWrapper<B>) -> Self::Output {
        self.difference(other)
    }
}

// Repeat via % operator
impl<A: Shape> Rem<Vec3> for ShapeWrapper<A> {
    type Output = ShapeWrapper<Repeat<ShapeWrapper<A>>>;

    fn rem(self, period: Vec3) -> Self::Output {
        self.repeat(period)
    }
}
