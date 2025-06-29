use eframe::wgpu;
use std::mem::size_of;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexRaw {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
}

impl VertexRaw {
    pub fn new(position: [f32; 3], tex_coords: [f32; 2], normal: [f32; 3]) -> VertexRaw {
        VertexRaw {
            position,
            tex_coords,
            normal,
        }
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
