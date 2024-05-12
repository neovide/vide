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
