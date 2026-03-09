struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct ShadowUniform {
    light_view_proj: mat4x4<f32>,
    light_direction: vec4<f32>,
    params: vec4<f32>,
};

struct TransformationRaw {
    model: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> transformation: TransformationRaw;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(2) @binding(1)
var s_diffuse: sampler;

@group(3) @binding(0)
var<uniform> shadow_uniform: ShadowUniform;

@group(3) @binding(1)
var shadow_map: texture_depth_2d;

@group(3) @binding(2)
var shadow_sampler: sampler_comparison;

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
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = in.tex_coords;
    out.normal = normalize((transformation.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_position = (transformation.model * vec4<f32>(in.position, 1.0)).xyz;
    out.clip_position = camera.view_proj * vec4<f32>(out.world_position, 1.0);
    return out;
}

@vertex
fn vs_main_outline(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = in.tex_coords;
    out.normal = normalize((transformation.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_position = (transformation.model * vec4<f32>(in.position, 1.0)).xyz;
    let extruded_position = in.position + in.normal * 0.02;
    out.clip_position = camera.view_proj * transformation.model * vec4<f32>(extruded_position, 1.0);
    return out;
}

@vertex
fn vs_shadow(in: VertexInput) -> @builtin(position) vec4<f32> {
    let world_position = transformation.model * vec4<f32>(in.position, 1.0);
    return camera.view_proj * world_position;
}

fn base_lighting(in: VertexOutput, light_direction: vec3<f32>) -> vec3<f32> {
    let albedo = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let normal = normalize(in.normal);
    let view_direction = normalize(-in.world_position);
    let half_vector = normalize(-light_direction + view_direction);

    let ambient_strength = 0.65;
    let diffuse_strength = 0.65;
    let specular_strength = 0.60;
    let shininess = 32.0;

    let lambert = max(dot(normal, -light_direction), 0.0);
    let specular = pow(max(dot(normal, half_vector), 0.0), shininess);
    let lighting = ambient_strength + diffuse_strength * lambert + specular_strength * specular;

    return albedo.rgb * lighting;
}

fn get_shadow_factor(in: VertexOutput, normal: vec3<f32>, light_direction: vec3<f32>) -> f32 {
    let light_clip_position = shadow_uniform.light_view_proj * vec4<f32>(in.world_position, 1.0);
    let projected = light_clip_position.xyz / light_clip_position.w;
    let uv = projected.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);

    if (projected.z < 0.0 || projected.z > 1.0 || uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return 1.0;
    }

    let base_bias = shadow_uniform.params.x;
    let ndotl = max(dot(normal, -light_direction), 0.0);
    let bias = max(base_bias * (1.0 - ndotl), base_bias * 0.25);
    return textureSampleCompare(shadow_map, shadow_sampler, uv, projected.z - bias);
}

@fragment
fn fs_unlit(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let light_direction = normalize(vec3<f32>(-0.45, -1.0, -0.35));
    return vec4<f32>(base_lighting(in, light_direction), albedo.a);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let normal = normalize(in.normal);
    let light_direction = normalize(shadow_uniform.light_direction.xyz);
    let view_direction = normalize(-in.world_position);
    let half_vector = normalize(-light_direction + view_direction);

    let ambient_strength = 0.65;
    let diffuse_strength = 0.65;
    let specular_strength = 0.60;
    let shininess = 32.0;

    let lambert = max(dot(normal, -light_direction), 0.0);
    let specular = pow(max(dot(normal, half_vector), 0.0), shininess);
    let direct = diffuse_strength * lambert + specular_strength * specular;
    let shadow_visibility = get_shadow_factor(in, normal, light_direction);
    let shadow_strength = shadow_uniform.params.y;
    let visible_direct = direct * ((1.0 - shadow_strength) + shadow_strength * shadow_visibility);
    return vec4<f32>(albedo.rgb * (ambient_strength + visible_direct), albedo.a);
}