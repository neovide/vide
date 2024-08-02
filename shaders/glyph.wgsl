#include "common.wgsl"

struct InstancedGlyph {
    bottom_left: vec2<f32>,
    atlas_top_left: vec2<f32>,
    atlas_size: vec2<f32>,
    kind: u32,
    color: vec4<f32>,
}

var<push_constant> constants: ShaderConstants;

@group(0) @binding(0) var<storage> glyphs: array<InstancedGlyph>;
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
    let instance = glyphs[instance_index];
    let vertex_pixel_pos = instance.bottom_left + (unit_vertex_pos - vec2(0., 1.)) * instance.atlas_size;
    
    let final_position = vec2(0.0, 2.0) + vertex_pixel_pos / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0;

    var out: VertexOutput;
    out.position = vec4(final_position, 0.0, 1.0);
    out.instance_index = instance_index;
    out.atlas_position = instance.atlas_top_left / constants.atlas_size
        + unit_vertex_pos * instance.atlas_size / constants.atlas_size;
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(0) @second_blend_source blend: vec4<f32>,
}

@fragment
fn frag(
    vertex_output: VertexOutput,
) -> FragmentOutput {
    let instance = glyphs[vertex_output.instance_index];
    let mask_color = textureSample(mask, texture_sampler, vertex_output.position.xy / constants.surface_size);
    let atlas_color = textureSample(atlas, texture_sampler, vertex_output.atlas_position);
    let text_color = instance.color;

    var out: FragmentOutput;
    if (instance.kind == 0 || instance.kind == 1) {
        // Blend equation should be 
        //     src_factor: BlendFactor::One
        //     dst_factor: BlendFactor::OneMinusSrc1
        //     operation: BlendOperation::Add
        //
        // The resulting color will be
        //     1.0 * text_color * atlas_color + (1.0 - text_color.w * atlas_color) * destination
        out.color = text_color * atlas_color;
        out.blend = text_color.w * atlas_color;
    } else {
        // We cheat a little to get the above blend factor
        // to work for full color glyphs.
        //
        // The resulting color will be
        //     1.0 * atlas_color + (1.0 - test_color.w * atlas_color.w * 1.0)
        //
        // For non transparent parts of the map, this
        // results in 1.0 * atlas_color + 0.0 or just
        // atlas_color
        //
        // For transparent parts of the map, this results
        // in 1.0 * atlas_color - 1.0 or just zero.
        out.color = atlas_color;
        out.blend = text_color.w * atlas_color.w * vec4(1.0);
    }

    out.color.w *= mask_color.w;
    out.blend.w *= mask_color.w;

    return out;
}
