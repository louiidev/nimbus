struct View {
    view_proj: mat4x4<f32>
};
@group(0) @binding(0)
var<uniform> view: View;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>
};

@vertex
fn vertex(
    obj_vert: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view.view_proj * vec4<f32>(obj_vert.position, 1.0);
    out.color = obj_vert.color;
    return out;
}


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {

    return in.clip_position;
}
