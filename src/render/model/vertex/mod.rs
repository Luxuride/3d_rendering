use cgmath::Vector3;
use vertex_raw::VertexRaw;

pub mod vertex_raw;

pub struct Vertex {
    pub position: Vector3<f32>
}

impl Vertex {
    pub fn to_raw(&self) -> VertexRaw {
        VertexRaw {
            position: self.position.into(),
        }
    }
}