struct Camera {
    view_proj: mat4x4<f32>,
}

struct Object {
    model: mat4x4<f32>,
    model_inv: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> object: Object;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
}


@vertex
fn vs_main(vtx_in: VertexIn) -> VertexOut {
    let position = camera.view_proj * object.model * vec4(vtx_in.position, 1.0);
    let normal = (vec4(vtx_in.normal, 1.0) * object.model_inv).xyz;
    return VertexOut(position, normal);
    // let x = f32(i32(in_vertex_index) - 1);
    // let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    // return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main(frag_in: VertexOut) -> @location(0) vec4<f32> {
    return vec4(abs(frag_in.normal), 1.0);
}
