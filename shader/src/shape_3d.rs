use core::ops::{Add, Mul, Sub, Rem};

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::glam::{vec3, Vec3, Quat};

pub trait Shape3D: Copy {
    fn distance(self, position: Vec3) -> f32;

    fn translate(self, translation: Vec3) -> Shape3DWrapper<Translation<Self>> {
        Shape3DWrapper(Translation(self, translation))
    }

    fn rotate(self, rotation: Quat) -> Shape3DWrapper<Rotation<Self>> {
        Shape3DWrapper(Rotation(self, rotation))
    }

    fn repeat(self, period: Vec3) -> Shape3DWrapper<Repeat<Self>> {
        Shape3DWrapper(Repeat(self, period))
    }

    fn scale(self, scale: f32) -> Shape3DWrapper<Scale<Self>> {
        Shape3DWrapper(Scale(self, scale))
    }

    fn union<S>(self, other: S) -> Shape3DWrapper<Union<Self, S>> {
        Shape3DWrapper(Union(self, other))
    }

    fn smooth_union<S>(self, other: S, amount: f32) -> Shape3DWrapper<SmoothUnion<Self, S>> {
        Shape3DWrapper(SmoothUnion(self, other, amount))
    }

    fn intersect<S>(self, other: S) -> Shape3DWrapper<Intersection<Self, S>> {
        Shape3DWrapper(Intersection(self, other))
    }

    fn difference<S>(self, other: S) -> Shape3DWrapper<Difference<Self, S>> {
        Shape3DWrapper(Difference(self, other))
    }
}

#[derive(Copy, Clone)]
pub struct Shape3DWrapper<A>(pub A);

impl<A: Shape3D> Shape3D for Shape3DWrapper<A> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(position)
    }
}

#[derive(Copy, Clone)]
pub struct Translation<A>(A, Vec3);

impl<A: Shape3D> Shape3D for Translation<A> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(position - self.1)
    }
}

#[derive(Copy, Clone)]
pub struct Rotation<A>(A, Quat);

impl<A: Shape3D> Shape3D for Rotation<A> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(self.1.inverse() * position)
    }
}

#[derive(Copy, Clone)]
pub struct Scale<A>(A, f32);

impl<A: Shape3D> Shape3D for Scale<A> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(position / self.1) * self.1
    }
}

#[derive(Copy, Clone)]
pub struct Repeat<A>(A, Vec3);

fn modulo(x: f32, m: f32) -> f32 {
    let x = x + m / 2.0;
    (((x % m) + m) % m) - m / 2.0
}

impl<A: Shape3D> Shape3D for Repeat<A> {
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

impl<A: Shape3D, B: Shape3D> Shape3D for Union<A, B> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(position).min(self.1.distance(position))
    }
}

#[derive(Copy, Clone)]
pub struct SmoothUnion<A, B>(A, B, f32);

impl<A: Shape3D, B: Shape3D> Shape3D for SmoothUnion<A, B> {
    fn distance(self, position: Vec3) -> f32 {
        let distance_a = self.0.distance(position);
        let distance_b = self.1.distance(position);

        let h = (0.5 - (distance_b - distance_a).abs()).max(0.0) / 0.5;
        distance_a.min(distance_b) - h * h * 0.125
    }
}

#[derive(Copy, Clone)]
pub struct Intersection<A, B>(A, B);

impl<A: Shape3D, B: Shape3D> Shape3D for Intersection<A, B> {
    fn distance(self, position: Vec3) -> f32 {
        self.0.distance(position).max(self.1.distance(position))
    }
}

#[derive(Copy, Clone)]
pub struct Difference<A, B>(A, B);

impl<A: Shape3D, B: Shape3D> Shape3D for Difference<A, B> {
    fn distance(self, position: Vec3) -> f32 {
        (-self.1.distance(position)).max(self.0.distance(position))
    }
}

// Translate via + operator
impl<A: Shape3D> Add<Vec3> for Shape3DWrapper<A> {
    type Output = Shape3DWrapper<Translation<Shape3DWrapper<A>>>;

    fn add(self, amount: Vec3) -> Self::Output {
        self.translate(amount)
    }
}

// Rotation via * operator
impl<A: Shape3D> Mul<Quat> for Shape3DWrapper<A> {
    type Output = Shape3DWrapper<Rotation<Shape3DWrapper<A>>>;

    fn mul(self, rotation: Quat) -> Self::Output {
        self.rotate(rotation)
    }
}

// Scale via * operator
impl<A: Shape3D> Mul<f32> for Shape3DWrapper<A> {
    type Output = Shape3DWrapper<Scale<Shape3DWrapper<A>>>;

    fn mul(self, scale: f32) -> Self::Output {
        self.scale(scale)
    }
}

// Union via + operator
impl<A: Shape3D, B: Shape3D> Add<Shape3DWrapper<B>> for Shape3DWrapper<A> {
    type Output = Shape3DWrapper<Union<Shape3DWrapper<A>, Shape3DWrapper<B>>>;

    fn add(self, other: Shape3DWrapper<B>) -> Self::Output {
        self.union(other)
    }
}

// Intersection via * operator
impl<A: Shape3D, B: Shape3D> Mul<Shape3DWrapper<B>> for Shape3DWrapper<A> {
    type Output = Shape3DWrapper<Intersection<Shape3DWrapper<A>, Shape3DWrapper<B>>>;

    fn mul(self, other: Shape3DWrapper<B>) -> Self::Output {
        self.intersect(other)
    }
}

// Difference via - operator
impl<A: Shape3D, B: Shape3D> Sub<Shape3DWrapper<B>> for Shape3DWrapper<A> {
    type Output = Shape3DWrapper<Difference<Shape3DWrapper<A>, Shape3DWrapper<B>>>;

    fn sub(self, other: Shape3DWrapper<B>) -> Self::Output {
        self.difference(other)
    }
}

// Repeat via % operator
impl<A: Shape3D> Rem<Vec3> for Shape3DWrapper<A> {
    type Output = Shape3DWrapper<Repeat<Shape3DWrapper<A>>>;

    fn rem(self, period: Vec3) -> Self::Output {
        self.repeat(period)
    }
}
