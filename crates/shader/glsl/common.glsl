#define PI 3.1415926535897932384626433832795;
#define FRAC_2_SQRT_PI 1.12837916709551257389615890312154517;

struct ShaderConstants {
    vec2 surface_size;
    vec2 atlas_size;
    vec4 clip;
};

const vec2 UNIT_QUAD_VERTICES[6] = vec2[6](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0)
);
