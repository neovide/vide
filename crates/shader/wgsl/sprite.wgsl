struct SpriteVertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn sprite_vertex() -> SpriteVertexOutput
{
    var out: SpriteVertexOutput;
    out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    return out;
}


@fragment
fn sprite_fragment(in: SpriteVertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
