struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

struct Draw {
    color: vec4<f32>;
    src_rect: vec4<f32>;
    transform: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Draw;

[[group(1), binding(0)]]
var t: texture_2d<f32>;

[[group(2), binding(0)]]
var s: sampler;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec2<f32>,
    [[location(1)]] uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.transform * vec4<f32>(position, 0.0, 1.0);
    out.uv = mix(uniforms.src_rect.xy, uniforms.src_rect.zw, uv);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return uniforms.color * textureSample(t, s, in.uv);
}