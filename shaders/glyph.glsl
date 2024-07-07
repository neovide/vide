struct InstancedGlyph {
    vec2 bottom_left;
    vec2 atlas_top_left;
    vec2 atlas_size;
    int kind;
    // Need a float of padding here so that fields are all aligned to
    // 16 bytes in size. Vec2s are 8 bytes, Vec4s are 16 bytes.
    float _padding;
    vec4 color;
};
