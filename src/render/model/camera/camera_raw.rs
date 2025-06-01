use crate::render::model::camera::Camera;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct CameraRaw {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraRaw {
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
