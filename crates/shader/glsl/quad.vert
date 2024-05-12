#include "common.glsl"
#include "quad.glsl"

layout (set=0, binding=0) buffer readonly InstancedQuad quads[];
layout (push_constant) uniform ShaderConstants constants;

layout (location=0) out uint out_instance_index;

void main() {
    out_instance_index = gl_InstanceIndex;

    vec2 unit_vertex_pos = UNIT_QUAD_VERTICES[gl_VertexIndex];

    InstancedQuad quad = quads[gl_InstanceIndex];
    vec2 blur_extension = vec2(max(quad.blur, 0.0) * 3.0);
    vec2 vertex_pixel_pos =
        (quad.top_left - blur_extension) + unit_vertex_pos * (quad.size + blur_extension * 2.0);

    vec2 final_position =
        vec2(0.0, 2.0) + vertex_pixel_pos / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0;
    gl_Position = vec4(final_position, 0.0, 1.0);
}
