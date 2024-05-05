struct GlyphVertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn glyph_vertex() -> GlyphVertexOutput
{
    var out: GlyphVertexOutput;
    out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    return out;
}

@fragment
fn glyph_fragment(in: GlyphVertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
