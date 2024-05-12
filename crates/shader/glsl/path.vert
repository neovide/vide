#include "common.glsl"

layout (push_constant) uniform ShaderConstants constants;

layout (location=0) in vec4  in_color;
layout (location=1) in vec2  in_position;
layout (location=0) out vec4 out_color;

void main() {
    out_color = in_color;
    gl_Position = vec4((vec2(0., 2.) + in_position / constants.surface_size * vec2(1., -1.) * 2.0 - 1.0), 0.0, 1.0);
}
