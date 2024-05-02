use spirv_std::glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use core::f32::consts::PI;

use crate::shape_2d::*;
use crate::shape_3d::*;
use crate::primitives_2d::*;
use crate::primitives_3d::*;

const PHONE_WIDTH: f32 = 97.0;
const PHONE_HEIGHT: f32 = 145.0;
const PHONE_DEPTH: f32 = 6.0;

const MIN_THICKNESS: f32 = 0.6;

const OUTER_WIDTH: f32 = 100.0;
const INNER_WIDTH: f32 = 20.5;
const OUTER_HEIGHT: f32 = 148.0;
const INNER_HEIGHT: f32 = 49.5;
const OUTER_ROTATION: f32 = 0.0;
const INNER_ROTATION: f32 = -PI as f32 * 2.0 / 180.0;
const OUTER_RADIUS: f32 = 8.0;
const INNER_RADIUS: f32 = 5.25;
const OUTER_X: f32 = 0.0;
const INNER_X: f32 = -27.75;
const OUTER_Y: f32 = 0.0;
const INNER_Y: f32 = -36.25;
const OUTER_Z: f32 = 0.0;
const INNER_Z: f32 = 3.15;
const BASE_THICKNESS: f32 = 6.0;
const LIGHT_X: f32 = 11.5;
const LIGHT_Y: f32 = -47.5;
const LIGHT_WIDTH: f32 = 4.0;
const LIGHT_EXPANDED_WIDTH: f32 = 10.0;
const LIGHT_HEIGHT: f32 = 9.0;
const LIGHT_EXPANDED_HEIGHT: f32 = LIGHT_HEIGHT + LIGHT_EXPANDED_WIDTH - LIGHT_WIDTH;
const LIGHT_RADIUS: f32 = (LIGHT_WIDTH / 2.0) * 0.9;
const LIGHT_EXPANDED_RADIUS: f32 = (LIGHT_EXPANDED_WIDTH / 2.0) * 0.9;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[derive(Copy, Clone)]
pub struct ContouredBack();

impl Shape3D for ContouredBack {
    fn distance(self, position: Vec3) -> f32 {
        let mut distance = core::f32::MAX;
        let mut i = 0.0;
        while i <= 1.0 {
            let radius = lerp(OUTER_RADIUS, INNER_RADIUS, i);
            let dimensions = vec2(
                lerp(OUTER_WIDTH, INNER_WIDTH, i) - radius,
                lerp(OUTER_HEIGHT, INNER_HEIGHT, i) - radius,
            );
            let rect = rectangle(dimensions).expand(radius);

            let slice_thickness = 1.0 / (INNER_Z - OUTER_Z);
            let slice = rect.extrude(slice_thickness);

            let translation = vec3(
                lerp(OUTER_X, INNER_X, i),
                lerp(OUTER_Y, INNER_Y, i),
                lerp(OUTER_Z, INNER_Z, i) * 5.0,
            );
            let slice = slice.translate(translation);

            let rotation = lerp(OUTER_ROTATION, INNER_ROTATION, i);
            let slice = slice.rotate(Quat::from_rotation_y(rotation));

            distance = distance.min(slice.distance(position));
            i += 1.0 / 20.0;
        }
        distance
    }
}

pub fn model() -> impl Shape3D {
    ContouredBack().rotate(Quat::from_rotation_x(PI)) 
}
