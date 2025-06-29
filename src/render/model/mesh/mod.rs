pub mod axis;
pub mod cube;

use crate::render::buffers::texture::texture_raw::TextureRaw;
use crate::render::buffers::transform::Transform;
use crate::render::buffers::vertex::vertex_raw::VertexRaw;
use crate::render::model::Model;
use crate::render::model::material::Material;
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use eframe::wgpu::{Device, Queue};

#[derive(Default)]
pub struct MeshBuilder {
    vertices: Vec<VertexRaw>,
    indices: Vec<u32>,
    material: usize,
}

impl MeshBuilder {
    pub fn vertices(mut self, vertices: Vec<VertexRaw>) -> Self {
        self.vertices = vertices;
        self
    }
    pub fn indices(mut self, indices: Vec<u32>) -> Self {
        self.indices = indices;
        self
    }
    pub fn material(mut self, material: usize) -> Self {
        self.material = material;
        self
    }
    pub fn build(self, device: &wgpu::Device) -> Mesh {
        Mesh::new(device, self.vertices, self.indices, self.material)
    }
}

impl From<tobj::Model> for MeshBuilder {
    fn from(m: tobj::Model) -> Self {
        let vertices = (0..m.mesh.positions.len() / 3)
            .map(|i| {
                if m.mesh.normals.is_empty() {
                    VertexRaw::new(
                        [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                        [0.0, 0.0, 0.0],
                    )
                } else {
                    VertexRaw::new(
                        [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        [
                            m.mesh.texcoords.get(i * 2).copied().unwrap_or(0.0),
                            1.0 - m.mesh.texcoords.get(i * 2 + 1).copied().unwrap_or(0.0),
                        ],
                        [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ],
                    )
                }
            })
            .collect::<Vec<_>>();
        let indices = m.mesh.indices;
        let res = Self::default().vertices(vertices).indices(indices);
        let material = m.mesh.material_id;
        if let Some(material) = material {
            return res.material(material);
        }
        res
    }
}

#[derive(Clone)]
pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    pub material: usize,
    vertices: Vec<VertexRaw>,
    indices: Vec<u32>,
}

impl Mesh {
    pub fn new(
        device: &Device,
        vertices: Vec<VertexRaw>,
        indices: Vec<u32>,
        material: usize,
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
        let index_count = indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            index_count,
            material,
            vertices,
            indices,
        }
    }
    pub fn get_num_indices(&self) -> u32 {
        self.index_count
    }
    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }
    pub fn get_index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }
    pub fn get_material(&self) -> usize {
        self.material
    }

    pub fn get_vertices(&self) -> &[VertexRaw] {
        &self.vertices
    }

    pub fn get_indices(&self) -> &[u32] {
        &self.indices
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_model(
        self,
        device: &Device,
        queue: &Queue,
        color: (f32, f32, f32),
        transform: Transform,
    ) -> Model {
        let texture = TextureRaw::from_color(device, queue, color, "color_texture").unwrap();
        let material = Material {
            diffuse_bind_group: texture.diffuse_bind_group(device),
            diffuse_texture: texture,
        };
        Model::new(device, vec![self], vec![material], transform)
    }
}
