@group(0) @binding(0)
var<uniform> u_transform: mat4x4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let positions: array<vec3<f32>, 16> = array<vec3<f32>, 16>(
        vec3<f32>( 0.0,  0.5,  0.0), vec3<f32>(-0.5, -0.5, -0.5), // 0-1
        vec3<f32>( 0.0,  0.5,  0.0), vec3<f32>( 0.5, -0.5, -0.5), // 0-2
        vec3<f32>( 0.0,  0.5,  0.0), vec3<f32>( 0.5, -0.5,  0.5), // 0-3
        vec3<f32>( 0.0,  0.5,  0.0), vec3<f32>(-0.5, -0.5,  0.5), // 0-4

        vec3<f32>(-0.5, -0.5, -0.5), vec3<f32>( 0.5, -0.5, -0.5), // 1-2
        vec3<f32>( 0.5, -0.5, -0.5), vec3<f32>( 0.5, -0.5,  0.5), // 2-3
        vec3<f32>( 0.5, -0.5,  0.5), vec3<f32>(-0.5, -0.5,  0.5), // 3-4
        vec3<f32>(-0.5, -0.5,  0.5), vec3<f32>(-0.5, -0.5, -0.5)  // 4-1
    );

    let pos = positions[in_vertex_index];
    return u_transform * vec4<f32>(pos, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 1.0, 1.0); // Cyan wireframe
}