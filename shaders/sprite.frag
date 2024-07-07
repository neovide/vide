#include "common.glsl"
#include "sprite.glsl"

layout(push_constant) uniform ShaderConstants constants;

layout (set=0, binding=0) buffer readonly InstancedSprite sprites[];
layout (set=0, binding=1) uniform texture2D atlas;
layout (set=1, binding=1) uniform texture2D mask;
layout (set=1, binding=2) uniform sampler texture_sampler;

layout (location=0) in uint in_instance_index;
layout (location=1) in vec2 in_atlas_position;

out vec4 out_color;

void main() {
    InstancedSprite instance = sprites[in_instance_index];
    vec4 image_color = texture(sampler2D(atlas, texture_sampler), in_atlas_position);
    vec4 mask_color = texture(sampler2D(mask, texture_sampler), gl_FragCoord.xy / constants.surface_size);
    out_color = instance.color * image_color;
    out_color.w *= mask_color.w;
}
