#include "sprite.glsl"

layout (set=0, binding=0) buffer readonly InstancedSprite sprites[];
layout (set=0, binding=1) uniform texture2D atlas;
layout (set=1, binding=1) uniform sampler atlas_sampler;

layout (location=0) in uint in_instance_index;
layout (location=1) in vec2 in_atlas_position;

out vec4 out_color;

void main() {
    InstancedSprite instance = sprites[in_instance_index];
    vec4 image_color = texture(sampler2D(atlas, atlas_sampler), in_atlas_position);
    out_color = instance.color * image_color;
}
