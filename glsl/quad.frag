#include "common.glsl"
#include "quad.glsl"

layout (push_constant) uniform ShaderConstants constants;

layout (set=0, binding=0) buffer readonly InstancedQuad quads[];
layout (set=1, binding=0) uniform texture2D surface;
layout (set=1, binding=1) uniform sampler texture_sampler;

layout (location=0) in uint in_instance_index;

layout (location=0) out vec4 out_color;


float quad_distance(InstancedQuad quad, vec2 point) {
    vec2 half_size = vec2(quad.size / 2.0 - quad.corner_radius);
    vec2 relative_point = point - (quad.top_left + quad.size / 2.0);
    vec2 d = abs(relative_point) - half_size;
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0) - quad.corner_radius;
}


float compute_erf7(float x){
    float x = x * FRAC_2_SQRT_PI;
    float xx = x * x;
    x += (0.24295 + (0.03395 + 0.0104 * xx) * xx) * (x * xx);
    return x / sqrt(1.0 + x * x);
}

void main() {
    InstancedQuad quad = quads[in_instance_index];

    float distance = quad_distance(quad, gl_FragCoord.xy);
    if (quad.blur > 0.0) {
        // Blurs the quad edge. Good for shadows.
        float min_edge = min(quad.size.x, quad.size.y);
        float inverse_blur = 1.0 / quad.blur;
        float scale = 0.5
            * compute_erf7(quad.blur * 0.5 * (max(quad.size.x, quad.size.y) - 0.5 * quad.corner_radius));
        float alpha = scale
            * (compute_erf7(inverse_blur * (min_edge + distance))
                - compute_erf7(inverse_blur * distance));
        out_color = quad.color;
        out_color.w *= alpha;
    } else {
        if (distance <= 0.0) {
            if (quad.blur < 0.0) {
                // Internal box blur sampled from background
                // Blur the quad background by sampling surrounding pixels
                // and averaging them using a dumb box blur.
                vec4 blurred_background = vec4(0.0);
                int blur = int(-quad.blur);
                int kernel_radius = abs(blur) - 1;
                float weight = 1.0 / pow((abs(kernel_radius) * 2 + 1), 2);
                for (int y=-kernel_radius;y<=kernel_radius;y++) {
                    for (int x=-kernel_radius;x<=kernel_radius;x++) {
                        vec2 offset = vec2(x, y);
                        vec2 sample_pos = (gl_FragCoord.xy + offset) / constants.surface_size;
                        vec4 sampled = texture(sampler2D(surface, texture_sampler), sample_pos);
                        blurred_background += sampled * weight;
                    }
                }

                float alpha = quad.color.w;
                out_color =
                    blurred_background * (1.0 - alpha) + vec4(quad.color.xyz * alpha, alpha);
            } else {
                out_color = quad.color;
            }
        }
    }
}
