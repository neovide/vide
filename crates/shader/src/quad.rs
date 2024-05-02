use glam::*;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::{image::Image2d, spirv, Sampler};

use crate::ShaderConstants;

const UNIT_QUAD_VERTICES: [Vec2; 6] = [
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0),
];

#[derive(Copy, Clone)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(Debug, bytemuck::Pod, bytemuck::Zeroable, Default)
)]
#[repr(C, align(64))]
// An axis aligned quad supporting positioning, scaling, corner radius, and optionally an internal blur with
// the previous layer or an external blur for use with shadows.
pub struct InstancedQuad {
    pub color: Vec4,
    pub _padding: Vec4,
    pub top_left: Vec2,
    pub size: Vec2,
    pub __padding: Vec2,
    pub corner_radius: f32,
    // 0: no blur
    // <0: internal blur of the background with kernel radius `blur`
    // >0: external blur of quad edge with radius `blur`
    pub blur: f32,
}

impl InstancedQuad {
    fn distance(&self, point: Vec2) -> f32 {
        let half_size = self.size / 2.0 - self.corner_radius * Vec2::ONE;
        let relative_point = point - (self.top_left + self.size / 2.0);
        let d = relative_point.abs() - half_size;
        d.max(Vec2::ZERO).length() + d.max_element().min(0.0) - self.corner_radius
    }
}

#[spirv(vertex)]
pub fn vertex(
    #[spirv(instance_index)] instance_index: i32,
    #[spirv(vertex_index)] vert_index: i32,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] quads: &[InstancedQuad],
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(position, invariant)] out_position: &mut Vec4,
    out_instance_index: &mut i32,
) {
    *out_instance_index = instance_index;

    let unit_vertex_pos = UNIT_QUAD_VERTICES[vert_index as usize];

    let quad = quads[instance_index as usize];
    let blur_extension = quad.blur.max(0.0) * 3.0 * Vec2::ONE;
    let vertex_pixel_pos =
        (quad.top_left - blur_extension) + unit_vertex_pos * (quad.size + blur_extension * 2.0);

    let final_position =
        vec2(0.0, 2.0) + vertex_pixel_pos / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0;
    *out_position = final_position.extend(0.0).extend(1.0);
}

#[spirv(fragment)]
pub fn fragment(
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] quads: &[InstancedQuad],
    #[spirv(descriptor_set = 1, binding = 0)] surface: &Image2d,
    #[spirv(descriptor_set = 1, binding = 1)] sampler: &Sampler,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(flat)] instance_index: i32,
    #[spirv(frag_coord)] surface_position: Vec4,
    out_color: &mut Vec4,
) {
    let quad = quads[instance_index as usize];

    let distance = quad.distance(surface_position.xy());
    if quad.blur > 0.0 {
        // Blurs the quad edge. Good for shadows.
        let min_edge = quad.size.min_element();
        let inverse_blur = 1.0 / quad.blur;
        let scale = 0.5
            * compute_erf7(quad.blur * 0.5 * (quad.size.max_element() - 0.5 * quad.corner_radius));
        let alpha = scale
            * (compute_erf7(inverse_blur * (min_edge + distance))
                - compute_erf7(inverse_blur * distance));
        *out_color = quad.color;
        out_color.w *= alpha;
    } else {
        if distance <= 0.0 {
            if quad.blur < 0.0 {
                // Internal box blur sampled from background
                // Blur the quad background by sampling surrounding pixels
                // and averaging them using a dumb box blur.
                let mut blurred_background = Vec4::ZERO;
                let blur = -quad.blur as i32;
                let kernel_radius = blur.abs() - 1;
                let weight = 1.0 / ((kernel_radius.abs() * 2 + 1).pow(2) as f32);
                for y in -kernel_radius..=kernel_radius {
                    for x in -kernel_radius..=kernel_radius {
                        let offset = vec2(x as f32, y as f32);
                        let sample_pos = (surface_position.xy() + offset) / constants.surface_size;
                        let sample = surface.sample_by_lod(*sampler, sample_pos, 0.);
                        blurred_background += sample * weight;
                    }
                }

                let alpha = quad.color.w;
                *out_color =
                    blurred_background * (1.0 - alpha) + (quad.color.xyz() * alpha).extend(alpha);
            } else {
                *out_color = quad.color;
            }
        }
    }
}

pub fn compute_erf7(x: f32) -> f32 {
    let x = x * core::f32::consts::FRAC_2_SQRT_PI;
    let xx = x * x;
    let x = x + (0.24295 + (0.03395 + 0.0104 * xx) * xx) * (x * xx);
    x / (1.0 + x * x).sqrt()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quad_distance() {
        // Initialize an instanced quad
        let quad = InstancedQuad {
            corner_radius: 5.0,
            top_left: Vec2::new(10.0, 10.0),
            size: Vec2::new(40.0, 50.0),
            ..Default::default()
        };

        assert_eq!(quad.distance(vec2(20.0, 10.0)), 0.0);
        assert_eq!(quad.distance(vec2(20.0, 20.0)), -10.0);
        assert_eq!(quad.distance(vec2(20.0, 5.0)), 5.0);
        assert_eq!(quad.distance(vec2(5.0, 5.0)), 9.142136);
    }
}
