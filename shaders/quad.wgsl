#include "common.wgsl"

struct InstancedQuad {
    color: vec4<f32>,
    _padding1: vec4<f32>,
    top_left: vec2<f32>,
    size: vec2<f32>,
    _padding2: vec2<f32>,
    corner_radius: f32,
    blur: f32,
}

var<push_constant> constants: ShaderConstants;

@group(0) @binding(0) var<storage> quads: array<InstancedQuad>;
@group(1) @binding(0) var surface: texture_2d<f32>;
@group(1) @binding(1) var mask: texture_2d<f32>;
@group(1) @binding(2) var texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) instance_index: u32,
}

@vertex
fn vert(
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var UNIT_QUAD_VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2(0.0, 0.0),
        vec2(1.0, 0.0),
        vec2(1.0, 1.0),
        vec2(0.0, 0.0),
        vec2(1.0, 1.0),
        vec2(0.0, 1.0)
    );

    let unit_vertex_pos = UNIT_QUAD_VERTICES[vertex_index];
    let instance = quads[instance_index];

    let blur_extension = vec2(max(instance.blur, 0.0) * 3.0);
    let vertex_pixel_pos =
        (instance.top_left - blur_extension) + unit_vertex_pos * (instance.size + blur_extension * 2.0);
    let final_position =
        vec2(0.0, 2.0) + vertex_pixel_pos / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0;

    var out: VertexOutput;
    out.instance_index = instance_index;
    out.position = vec4(final_position, 0.0, 1.0);
    return out;
}

fn quad_distance(quad: InstancedQuad, point: vec2<f32>) -> f32 {
    let half_size = vec2(quad.size / 2.0 - quad.corner_radius);
    let relative_point = point - (quad.top_left + quad.size / 2.0);
    let d = abs(relative_point) - half_size;
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0) - quad.corner_radius;
}

fn compute_erf7(in: f32) -> f32 {
    var x = in * FRAC_2_SQRT_PI;
    let xx = x * x;
    x += (0.24295 + (0.03395 + 0.0104 * xx) * xx) * (x * xx);
    return x / sqrt(1.0 + x * x);
}

@fragment
fn frag(
    vertex_output: VertexOutput,
) -> @location(0) vec4<f32> {
    let instance = quads[vertex_output.instance_index];
    let mask_color = textureSample(mask, texture_sampler, vertex_output.position.xy / constants.surface_size);

    let distance = quad_distance(instance, vertex_output.position.xy);
    var result = vec4(0.0);
    if (instance.blur > 0.0) {
        let min_edge = min(instance.size.x, instance.size.y);
        let inverse_blur = 1.0 / instance.blur;

        let scale = 0.5
            * compute_erf7(instance.blur * 0.5 * (max(instance.size.x, instance.size.y) - 0.5 * instance.corner_radius));

        let alpha = scale
            * (compute_erf7(inverse_blur * (min_edge + distance))
                - compute_erf7(inverse_blur * distance));

        result = instance.color;
        result.w *= alpha;
    } else if (distance <= 0.0) {
        if (instance.blur < 0.0) {
            // Internal box blur sampled from background
            // Blur the quad background by sampling surrounding pixels
            // and averaging them using a dumb box blur.
            var blurred_background = vec4(0.0);
            let blur = i32(-instance.blur);
            let kernel_radius = abs(blur) - 1;
            let weight = 1.0 / pow((abs(f32(kernel_radius)) * 2.0 + 1.0), 2.0);
            for (var y=-kernel_radius;y<=kernel_radius;y++) {
                for (var x=-kernel_radius;x<=kernel_radius;x++) {
                    let offset = vec2(f32(x), f32(y));
                    let sample_pos = (vertex_output.position.xy + offset) / constants.surface_size;

                    let sampled = textureSample(surface, texture_sampler, sample_pos);
                    blurred_background += sampled * weight;
                }
            }

            let alpha = instance.color.w;
            result =
                blurred_background * (1.0 - alpha) + vec4(instance.color.xyz * alpha, alpha);
        } else {
            result = instance.color;
        }
    }

    result.w *= mask_color.w;
    return result;
}

