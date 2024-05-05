struct PathVertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn path_vertex() -> PathVertexOutput
{
    var out: PathVertexOutput;
    out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    return out;
}

@fragment
fn path_fragment(in: PathVertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
