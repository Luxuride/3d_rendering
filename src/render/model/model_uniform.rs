#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    model_matrix: [[f32; 4]; 4],
}
