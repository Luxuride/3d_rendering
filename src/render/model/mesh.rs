use crate::render::model::vertex::vertex_raw::VertexRaw;
use cgmath::{Quaternion, Vector3, Zero};
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use crate::render::model::transform::Transform;
use crate::render::model::transform::transform_raw::TransformRaw;

pub struct MeshBuilder {
    pub vertices: Vec<VertexRaw>,
    pub indices: Vec<u16>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
            position: Vector3::zero(),
            rotation: Quaternion::zero(),
        }
    }
}

impl MeshBuilder {
    pub fn vertices(mut self, vertices: Vec<VertexRaw>) -> Self {
        self.vertices = vertices;
        self
    }
    pub fn indices(mut self, indices: Vec<u16>) -> Self {
        self.indices = indices;
        self
    }
    pub fn position(mut self, position: Vector3<f32>) -> Self {
        self.position = position;
        self
    }
    pub fn rotation(mut self, rotation: Quaternion<f32>) -> Self {
        self.rotation = rotation;
        self
    }
    pub fn build(self, device: &wgpu::Device) -> Mesh {
        Mesh::new(
            device,
            self.vertices,
            self.indices,
            Transform {
                position: self.position,
                rotation: self.rotation,
            },
        )
    }
}

pub struct Mesh {
    vertices: Vec<VertexRaw>,
    vertex_buffer: wgpu::Buffer,
    indices: Vec<u16>,
    index_buffer: wgpu::Buffer,
    transform: Transform,
    transform_buffer: wgpu::Buffer,
    transform_bind_group: wgpu::BindGroup,
}

impl Mesh {
    pub fn new(
        device: &wgpu::Device,
        vertices: Vec<VertexRaw>,
        indices: Vec<u16>,
        transform: Transform,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&transform.to_raw().model),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let transform_bind_group_layout = TransformRaw::transform_bind_group_layout(device);
        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
            label: Some("per_mesh_transform_bind_group"),
        });

        Self {
            vertex_buffer,
            index_buffer,
            transform,
            transform_buffer,
            vertices,
            indices,
            transform_bind_group,
        }
    }
    pub fn get_num_indices(&self) -> u32 {
        self.indices.len() as u32
    }
    pub fn get_vertices(&self) -> &[VertexRaw] {
        &self.vertices
    }
    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }
    pub fn get_indices(&self) -> &[u16] {
        &self.indices
    }
    pub fn get_index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }
    pub fn get_transform(&self) -> &Transform {
        &self.transform
    }
    pub fn get_transform_buffer(&self) -> &wgpu::Buffer {
        &self.transform_buffer
    }
    pub fn get_transform_bind_group(&self) -> &wgpu::BindGroup {
        &self.transform_bind_group
    }
    pub fn set_vertices(&mut self, vertices: Vec<VertexRaw>) {
        self.vertices = vertices;
    }
    pub fn set_indices(&mut self, indices: Vec<u16>) {
        self.indices = indices;
    }
}
