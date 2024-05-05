struct QuadVertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn quad_vertex() -> QuadVertexOutput
{
    var out: QuadVertexOutput;
    out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    return out;
}

@fragment
fn quad_fragment(in: QuadVertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
