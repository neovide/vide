#include "common.wgsl"

struct InstancedSprite {
    top_left: vec2<f32>,
    size: vec2<f32>,
    atlas_top_left: vec2<f32>,
    atlas_size: vec2<f32>,
    color: vec4<f32>,
}

var<push_constant> constants: ShaderConstants;

@group(0) @binding(0) var<storage> sprites: array<InstancedSprite>;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(1) @binding(1) var mask: texture_2d<f32>;
@group(1) @binding(2) var texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) instance_index: u32,
    @location(1) atlas_position: vec2<f32>,
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
    let instance = sprites[instance_index];
    let vertex_pixel_pos = instance.top_left + unit_vertex_pos * instance.size;

    let final_position =
        vec2(0.0, 2.0) + vertex_pixel_pos / constants.surface_size * vec2(1.0, -1.0) * 2.0 - 1.0;

    var out: VertexOutput;
    out.position = vec4(final_position, 0.0, 1.0);
    out.instance_index = instance_index;
    out.atlas_position = instance.atlas_top_left / constants.atlas_size
        + unit_vertex_pos * instance.atlas_size / constants.atlas_size;
    return out;
}

@fragment
fn frag(
    vertex_output: VertexOutput,
) -> @location(0) vec4<f32> {
    let instance = sprites[vertex_output.instance_index];

    let atlas_color = textureSample(atlas, texture_sampler, vertex_output.atlas_position);
    let mask_color = textureSample(mask, texture_sampler, vertex_output.position.xy / constants.surface_size);

    var result = instance.color * atlas_color;
    result.w *= mask_color.w;
    return result;
}
