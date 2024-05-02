#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use spirv_std::glam::{vec2, Vec2, Vec3, Vec4};

pub fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn random(seed: f32) -> f32 {
    let value = seed.sin() * 1000000.0;
    value - value.floor()
}

pub fn random_2d(seed: Vec2) -> f32 {
    let value = seed.dot(vec2(12.9898, 78.233)).sin().abs() * 43758.5453123;
    value - value.floor()
}

pub fn noise(input: Vec2) -> f32 {
    let floor = input.floor();
    let fraction = input - floor;

    let a = random_2d(floor);
    let b = random_2d(floor + Vec2::X);
    let c = random_2d(floor + Vec2::Y);
    let d = random_2d(floor + Vec2::ONE);

    let u = fraction * fraction * (Vec2::splat(3.0) - 2.0 * fraction);

    mix(a, b, u.x) +
        (c - a) * u.y * (1.0 - u.x) +
        (d - b) * u.x * u.y
}

pub fn fbm(mut input: Vec2) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;

    for _ in 0..8 {
        value += amplitude * noise(input);
        input *= 2.0;
        amplitude *= 0.5;
    }

    value
}
