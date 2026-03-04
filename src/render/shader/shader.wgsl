// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct TransformationRaw {
    model: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> transformation: TransformationRaw;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = in.tex_coords;
    out.normal = normalize((transformation.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_position = (transformation.model * vec4<f32>(in.position, 1.0)).xyz;
    out.clip_position = camera.view_proj * transformation.model * vec4<f32>(in.position, 1.0);
    return out;
}

@vertex
fn vs_main_outline(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = in.tex_coords;
    out.normal = normalize((transformation.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_position = (transformation.model * vec4<f32>(in.position, 1.0)).xyz;
    // Inverted Hull Outline
    let extruded_position = in.position + in.normal * 0.02;
    out.clip_position = camera.view_proj * transformation.model * vec4<f32>(extruded_position, 1.0);
    return out;
}

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    let normal = normalize(in.normal);
    let light_direction = normalize(vec3<f32>(-0.45, -1.0, -0.35));
    let view_direction = normalize(-in.world_position);
    let half_vector = normalize(-light_direction + view_direction);

    let ambient_strength = 0.65;
    let diffuse_strength = 0.65;
    let specular_strength = 0.60;
    let shininess = 32.0;

    let lambert = max(dot(normal, -light_direction), 0.0);
    let specular = pow(max(dot(normal, half_vector), 0.0), shininess);
    let lighting = ambient_strength + diffuse_strength * lambert;
    let specular_lighting = specular_strength * specular;

    return vec4<f32>(albedo.rgb * (lighting + specular_lighting), albedo.a);
}