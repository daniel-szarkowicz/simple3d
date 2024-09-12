struct Camera {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
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
    @location(1) view_dir: vec3<f32>,
}


@vertex
fn vs_main(vtx_in: VertexIn) -> VertexOut {
    let world_position = object.model * vec4(vtx_in.position, 1.0);
    let position = camera.view_proj * world_position;
    let normal = (vec4(vtx_in.normal, 1.0) * object.model_inv).xyz;
    let view_dir = camera.position * world_position.w - world_position.xyz;
    return VertexOut(position, normal, view_dir);
}

@fragment
fn fs_main(frag_in: VertexOut) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(frag_in.normal);
    let view_dir = normalize(frag_in.view_dir);

    let diffuse = max(dot(normal, light_dir), 0.0);
    let ambient = max(dot(normal, view_dir), 0.0);

    let brightness = 0.7 * diffuse + 0.2 * ambient + 0.1;
    let color = vec3<f32>(0.5, 0.5, 0.0);
    return vec4(brightness * color, 1.0);
}
