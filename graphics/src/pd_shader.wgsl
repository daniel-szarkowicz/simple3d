struct Camera {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct InstanceIn {
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
    @location(8) model_inv_0: vec4<f32>,
    @location(9) model_inv_1: vec4<f32>,
    @location(10) model_inv_2: vec4<f32>,
    @location(11) model_inv_3: vec4<f32>,
    @location(12) color: vec3<f32>,
}

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) direction: vec3<f32>,
}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) view_dir: vec3<f32>,
    @location(2) color: vec3<f32>,
}


@vertex
fn vs_main(vtx_in: VertexIn, instance_in: InstanceIn) -> VertexOut {
    let model = mat4x4<f32>(
        instance_in.model_0,
        instance_in.model_1,
        instance_in.model_2,
        instance_in.model_3,
    );
    let model_inv = mat4x4<f32>(
        instance_in.model_inv_0,
        instance_in.model_inv_1,
        instance_in.model_inv_2,
        instance_in.model_inv_3,
    );

    let world_position = model * vec4(vtx_in.position, 1.0);
    let position = camera.view_proj * world_position;
    let direction = (vec4(vtx_in.direction, 1.0) * model_inv).xyz;
    let view_dir = camera.position * world_position.w - world_position.xyz;

    // let normal_perp = normalize(cross(view_dir, direction));
    // let normal = normalize(cross(direction, normal_perp));
    // use vector triple product to avoid double cross product
    let normal = normalize(
        dot(direction, direction) * view_dir
        - dot(direction, view_dir) * direction
    );
    return VertexOut(position, normal, view_dir, instance_in.color);
}

@fragment
fn fs_main(frag_in: VertexOut) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(frag_in.normal);
    let view_dir = normalize(frag_in.view_dir);

    let diffuse = max(dot(normal, light_dir), 0.0);
    let ambient = max(dot(normal, view_dir), 0.0);

    let brightness = 0.7 * diffuse + 0.2 * ambient + 0.1;
    return vec4(brightness * frag_in.color, 1.0);
}
