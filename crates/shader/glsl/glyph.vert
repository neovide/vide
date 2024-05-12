#include "common.glsl"
#include "glyph.glsl"

layout (push_constant) uniform ShaderConstants constants;

layout (set=0, binding=0) buffer readonly InstancedGlyph glyphs[];

layout (location=0) out uint out_instance_index;
layout (location=1) out vec2 out_atlas_position;

void main() {
    out_instance_index = gl_InstanceIndex;
    vec2 unit_vertex_pos = UNIT_QUAD_VERTICES[gl_VertexIndex];
    InstancedGlyph instance = glyphs[gl_InstanceIndex];
    vec2 vertex_pixel_pos =
        instance.bottom_left + (unit_vertex_pos - vec2(0., 1.)) * instance.atlas_size;

    vec2 final_position =
        vec2(0.0, 2.0) + vertex_pixel_pos / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0;
    gl_Position = vec4(final_position, 0.0, 1.0);

    out_atlas_position = instance.atlas_top_left / constants.atlas_size
        + unit_vertex_pos * instance.atlas_size / constants.atlas_size;
}
