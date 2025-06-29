#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct CameraRaw {
    view_proj: [[f32; 4]; 4],
}

impl CameraRaw {
    pub fn new(view_proj: [[f32; 4]; 4]) -> Self {
        Self { view_proj }
    }

    // Getter methods
    pub fn get_view_proj(&self) -> &[[f32; 4]; 4] {
        &self.view_proj
    }

    pub fn get_view_proj_mut(&mut self) -> &mut [[f32; 4]; 4] {
        &mut self.view_proj
    }
}
