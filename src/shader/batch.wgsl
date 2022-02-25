struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1), interpolate(flat)]] idx: u32;
};

struct Draw {
    color: vec4<f32>;
    src_rect: vec4<f32>;
    transform: mat4x4<f32>;
};

struct Draws {
    draws: [[stride(96)]] array<Draw>;
};

[[group(0), binding(0)]]
var<storage, read> instances: Draws;

[[group(1), binding(0)]]
var t: texture_2d<f32>;

[[group(2), binding(0)]]
var s: sampler;

[[stage(vertex)]]
fn vs_main(
    [[builtin(instance_index)]] in_instance_index: u32,
    [[location(0)]] position: vec2<f32>,
    [[location(1)]] uv: vec2<f32>,
) -> VertexOutput {
    var instance = instances.draws[in_instance_index];

    var out: VertexOutput;
    out.position = instance.transform * vec4<f32>(position, 0.0, 1.0);
    out.uv = mix(instance.src_rect.xy, instance.src_rect.zw, uv);
    out.idx = in_instance_index;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return instances.draws[in.idx].color * textureSample(t, s, in.uv);
}