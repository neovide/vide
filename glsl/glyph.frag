#include "common.glsl"
#include "glyph.glsl"

// Reference: https://github.com/servo/webrender/blob/main/webrender/doc/text-rendering.md

layout(push_constant) uniform ShaderConstants constants;

layout(set = 0, binding = 0) buffer readonly InstancedGlyph glyphs[];
layout(set = 0, binding = 1) uniform texture2D atlas;
layout(set = 1, binding = 0) uniform texture2D surface;
layout(set = 1, binding = 1) uniform texture2D mask;
layout(set = 1, binding = 2) uniform sampler texture_sampler;

layout(location = 0) in uint in_instance_index;
layout(location = 1) in vec2 in_atlas_position;

out vec4 out_color;

void main() {
    InstancedGlyph glyph = glyphs[in_instance_index];
    vec4 surface_color = texture(sampler2D(surface, texture_sampler), gl_FragCoord.xy / constants.surface_size);
    vec4 mask_color = texture(sampler2D(mask, texture_sampler), gl_FragCoord.xy / constants.surface_size);
    vec4 atlas_color = texture(sampler2D(atlas, texture_sampler), in_atlas_position);
    vec4 text_color = glyph.color;

    if (glyph.kind == 0 || glyph.kind == 1) {
        out_color = text_color * atlas_color + (1.0 - text_color.w * atlas_color) * surface_color;
        if (atlas_color.x != 0.0 || atlas_color.y != 0.0 || atlas_color.z != 0.0) {
            out_color.w = 1.0;
        } else {
            out_color.w = 0.0;
        }
    } else {
        out_color = atlas_color;
    }

    out_color.w *= mask_color.w;
}
