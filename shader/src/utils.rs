#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use spirv_std::glam::*;

pub fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn abs(vec: Vec3) -> Vec3 {
    vec3(vec.x.abs(), vec.y.abs(), vec.z.abs())
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

pub fn turbulence(mut input: Vec2) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;

    for _ in 0..10 {
        value += amplitude * noise(input).abs();
        input *= 2.0;
        amplitude *= 0.5;
    }

    value
}

pub fn animated_fbm(mut input: Vec2, time: f32) -> f32 {
    let q = vec2(
        fbm(input + Vec2::splat(0.01 * time)),
        fbm(input + Vec2::ONE));

    let r = vec2(
        fbm(input + 1.0 * q + vec2(1.7, 9.2) + Vec2::splat(0.15 * time)),
        fbm(input + 1.0 * q + vec2(8.3, 2.8) + Vec2::splat(0.126 * time)));

    fbm(input + r)
}
