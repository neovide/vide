#include "common.wgsl"

struct InstancedBlur {
    color: vec4<f32>,
    _padding1: vec4<f32>,
    top_left: vec2<f32>,
    size: vec2<f32>,
    _padding2: vec2<f32>,
    corner_radius: f32,
    blur: f32,
}

var<push_constant> constants: ShaderConstants;

@group(0) @binding(0) var<storage> blurs: array<InstancedBlur>;
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
    let instance = blurs[instance_index];

    let vertex_pixel_pos =
        instance.top_left + unit_vertex_pos * instance.size;
    let final_position =
        vec2(0.0, 2.0) + min(vertex_pixel_pos, constants.surface_size) 
        / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0;

    var out: VertexOutput;
    out.instance_index = instance_index;
    out.position = vec4(final_position, 0.0, 1.0);
    return out;
}

fn blur_distance(blur: InstancedBlur, point: vec2<f32>) -> f32 {
    let half_size = vec2(blur.size / 2.0 - blur.corner_radius);
    let relative_point = point - (blur.top_left + blur.size / 2.0);
    let d = abs(relative_point) - half_size;
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0) - blur.corner_radius;
}

@fragment
fn frag(
    vertex_output: VertexOutput,
) -> @location(0) vec4<f32> {
    let instance = blurs[vertex_output.instance_index];
    let mask_color = textureSample(mask, texture_sampler, vertex_output.position.xy / constants.surface_size);

    let distance = blur_distance(instance, vertex_output.position.xy);
    var result = vec4(0.0);
    if (distance <= 0.0) {
        // Internal box blur sampled from background
        // Blur the quad background by sampling surrounding pixels
        // and averaging them using a dumb box blur.
        var blurred_background = vec4(0.0);
        let blur = i32(abs(instance.blur));
        let kernel_radius = blur - 1;
        let weight = 1.0 / pow((f32(kernel_radius) * 2.0 + 1.0), 2.0);
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
    }

    result.w *= mask_color.w;
    return result;
}

