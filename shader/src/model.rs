#[cfg(not(target_arch = "spirv"))]
use bevy_reflect::Reflect;
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

const BASE_THICKNESS: f32 = 6.0;
const LIGHT_X: f32 = 11.5;
const LIGHT_Y: f32 = -47.5;
const LIGHT_WIDTH: f32 = 4.0;
const LIGHT_EXPANDED_WIDTH: f32 = 10.0;
const LIGHT_HEIGHT: f32 = 9.0;
const LIGHT_EXPANDED_HEIGHT: f32 = LIGHT_HEIGHT + LIGHT_EXPANDED_WIDTH - LIGHT_WIDTH;
const LIGHT_RADIUS: f32 = (LIGHT_WIDTH / 2.0) * 0.9;
const LIGHT_EXPANDED_RADIUS: f32 = (LIGHT_EXPANDED_WIDTH / 2.0) * 0.9;


#[cfg_attr(not(target_arch = "spirv"), derive(Reflect))]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ModelConstants {
    pub outer_width: f32,
    pub inner_width: f32,
    pub outer_height: f32,
    pub inner_height: f32,
    pub outer_rotation: f32,
    pub inner_rotation: f32,
    pub outer_radius: f32,
    pub inner_radius: f32,
    pub outer_x: f32,
    pub inner_x: f32,
    pub outer_y: f32,
    pub inner_y: f32,
    pub outer_z: f32,
    pub inner_z: f32,
}

impl Default for ModelConstants {
    fn default() -> Self {
        Self {
            outer_width: 100.0,
            inner_width: 20.5,
            outer_height: 148.0,
            inner_height: 49.5,
            outer_rotation: 0.0,
            inner_rotation: -PI as f32 * 2.0 / 180.0,
            outer_radius: 8.0,
            inner_radius: 5.25,
            outer_x: 0.0,
            inner_x: -27.75,
            outer_y: 0.0,
            inner_y: -36.25,
            outer_z: 0.0,
            inner_z: 3.15,
        } 
    }
}  

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[derive(Copy, Clone)]
pub struct ContouredBack {
    constants: ModelConstants
}

impl Shape3D for ContouredBack {
    fn distance(self, position: Vec3) -> f32 {
        let ModelConstants {
            outer_width,
            inner_width,
            outer_height,
            inner_height,
            outer_rotation,
            inner_rotation,
            outer_radius,
            inner_radius,
            outer_x,
            inner_x,
            outer_y,
            inner_y,
            outer_z,
            inner_z,
        } = self.constants;

        let mut distance = core::f32::MAX;
        let mut i = 0.0;
        while i <= 1.0 {
            let radius = lerp(outer_radius, inner_radius, i);
            let dimensions = vec2(
                lerp(outer_width, inner_width, i) - radius,
                lerp(outer_height, inner_height, i) - radius,
            );
            let rect = rectangle(dimensions).expand(radius);

            let slice_thickness = 1.0 / (inner_z - outer_z);
            let slice = rect.extrude(slice_thickness);

            let translation = vec3(
                lerp(outer_x, inner_x, i),
                lerp(outer_y, inner_y, i),
                lerp(outer_z, inner_z, i) * 5.0,
            );
            let slice = slice.translate(translation);

            let rotation = lerp(outer_rotation, inner_rotation, i);
            let slice = slice.rotate(Quat::from_rotation_y(rotation));

            distance = distance.min(slice.distance(position));
            i += 1.0 / 20.0;
        }
        distance
    }
}

pub fn model(constants: ModelConstants) -> impl Shape3D {
    let back = ContouredBack { constants };
    back.rotate(Quat::from_rotation_x(PI)) 
}
