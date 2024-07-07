struct InstancedQuad {
    vec4 color;
    vec4 _padding;
    vec2 top_left;
    vec2 size;
    vec2 __padding;
    float corner_radius;
    // 0: no blur
    // <0: internal blur of the background with kernel radius `blur`
    // >0: external blur of quad edge with radius `blur`
    float blur;
};
