use crate::render::renderer::RendererRenderResources;
use cgmath::{Matrix4, Vector3};
use std::ops::Mul;

impl RendererRenderResources {
    pub fn move_x(&mut self, x: f32) {
        self.move_x_y_z(x, 0.0, 0.0);
    }

    pub fn move_y(&mut self, y: f32) {
        self.move_x_y_z(0.0, y, 0.0);
    }
    pub fn move_z(&mut self, z: f32) {
        self.move_x_y_z(0.0, 0.0, z);
    }
    pub fn move_x_y_z(&mut self, x: f32, y: f32, z: f32) {
        self.model_matrix = self
            .model_matrix
            .mul(Matrix4::from_translation(Vector3::new(x, y, z)));
    }
}
