struct Camera {
    view_proj: mat4x4<f32>,
}

struct Object {
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> object: Object;


@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    // @builtin(vertex_index) in_vertex_index: u32
) -> @builtin(position) vec4<f32> {
    let out_pos = camera.view_proj * object.model * vec4<f32>(position, 1.0);
    return out_pos;
    // let x = f32(i32(in_vertex_index) - 1);
    // let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    // return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.5, 1.0);
}
