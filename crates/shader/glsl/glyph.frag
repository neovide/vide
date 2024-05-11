#version 460

struct InstancedGlyph {
    vec2 bottom_left;
    vec2 atlas_top_left;
    vec2 atlas_size;
    // Need a Vec2 of padding here so that the first 4 fields
    // Are some multiple of 16 bytes in size.
    // Vec2s are 8 bytes, Vec4s are 16 bytes.
    vec2 _padding;
    vec4 color;
};

struct ShaderConstants {
    vec2 surface_size;
    vec2 atlas_size;
    vec4 clip;
};

layout (location=0) in uint in_instance_index;
layout (location=1) in vec2 in_atlas_position;

layout (push_constant) uniform ShaderConstants constants;
layout (set=0, binding=0) buffer readonly InstancedGlyph glyphs[];
layout (set=0, binding=1) uniform texture2D atlas;
layout (set=1, binding=0) uniform texture2D surface;
layout (set=1, binding=1) uniform sampler texture_sampler;

out vec4 out_color;

void main() {
    InstancedGlyph glyph = glyphs[in_instance_index];
    vec4 surface_color = texture(sampler2D(surface, texture_sampler), gl_FragCoord.xy / constants.surface_size);
    vec4 mask_color = texture(sampler2D(atlas, texture_sampler), in_atlas_position);
    out_color = glyph.color * mask_color + (1.0 - glyph.color.w * mask_color) * surface_color;
}
