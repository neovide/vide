#include "common.wgsl"

var<push_constant> constants: ShaderConstants;

@group(1) @binding(1) var mask: texture_2d<f32>;
@group(1) @binding(2) var texture_sampler: sampler;

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vert(
    @location(0) color: vec4<f32>,
    @location(1) position: vec2<f32>,
) -> VertexOutput
{
    var out: VertexOutput;
    out.color = color;
    out.position = vec4<f32>(
        vec2<f32>(0.0, 2.0) + position / constants.surface_size * vec2<f32>(1.0, -1.0) * 2.0 - 1.0,
        0.0, 1.0);
    return out;
}


@fragment
fn frag(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    let mask_color = textureSample(mask, texture_sampler, vertex_output.position.xy / constants.surface_size);
    var out = vertex_output.color;
    out.w *= mask_color.w;
    return out;
}
