#include "common.glsl"

layout(push_constant) uniform ShaderConstants constants;

layout(set = 1, binding = 1) uniform texture2D mask;
layout(set = 1, binding = 2) uniform sampler texture_sampler;

layout (location=0) in vec4 in_color;

layout (location=0) out vec4 out_color;

void main() {
    vec4 mask_color = texture(sampler2D(mask, texture_sampler), gl_FragCoord.xy / constants.surface_size);
    out_color = in_color;
    out_color.w *= mask_color.w;
}
