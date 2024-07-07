struct ShaderConstants {
    surface_size: vec2<f32>,
    atlas_size: vec2<f32>,
}

const UNIT_QUAD_VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0)
);
