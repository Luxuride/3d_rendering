use crate::render::model::mesh::Mesh;
use cgmath::Matrix4;
use eframe::wgpu::{BindGroup, Buffer};
use std::sync::Arc;

pub struct RenderableObject {
    pub mesh: Arc<Mesh>, // Arc allows sharing mesh data if multiple objects use the same base mesh
    pub model_bind_group: BindGroup,
    pub model_uniform_buffer: Buffer,
    pub transform: Matrix4<f32>, // The actual transformation matrix
}
