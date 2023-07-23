struct View {
    view_proj: mat4x4<f32>
};
@group(0) @binding(0)
var<uniform> view: View;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>
};

@vertex
fn vertex(
    obj_vert: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view.view_proj * vec4<f32>(obj_vert.position, 1.0);
    out.uv = obj_vert.uv;
    out.color = obj_vert.color;
    return out;
}

@group(1) @binding(0)
var obj_texture: texture_2d<f32>;
@group(1) @binding(1)
var obj_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(obj_texture, obj_sampler, in.uv);
    
    return in.color * color;
}
