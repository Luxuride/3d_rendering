pub mod axis;
pub mod cube;

use crate::render::buffers::color::color_raw::ColorRaw;
use crate::render::buffers::transform::transform_raw::TransformRaw;
use crate::render::buffers::transform::Transform;
use crate::render::buffers::vertex::vertex_raw::VertexRaw;
use cgmath::{Quaternion, Vector3, Zero};
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use eframe::wgpu::{include_wgsl, BindGroupLayout, ColorTargetState, Device, RenderPipeline};
use tobj::Model;

pub struct MeshBuilder {
    vertices: Vec<VertexRaw>,
    indices: Vec<u32>,
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
    material: usize,
    color: ColorRaw,
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
            position: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            material: 0,
            color: ColorRaw::new(1.0, 1.0, 1.0, 1.0),
        }
    }
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
    pub fn position(mut self, position: Vector3<f32>) -> Self {
        self.position = position;
        self
    }
    pub fn rotation(mut self, rotation: Quaternion<f32>) -> Self {
        self.rotation = rotation;
        self
    }
    pub fn material(mut self, material: usize) -> Self {
        self.material = material;
        self
    }
    pub fn color(mut self, color: ColorRaw) -> Self {
        self.color = color;
        self
    }
    pub fn scale(mut self, scale: Vector3<f32>) -> Self {
        self.scale = scale;
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
                scale: self.scale,
            },
            self.material,
            self.color,
        )
    }
}

impl From<Model> for MeshBuilder {
    fn from(m: Model) -> Self {
        let vertices = (0..m.mesh.positions.len() / 3)
            .map(|i| {
                if m.mesh.normals.is_empty() {
                    VertexRaw {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                        normal: [0.0, 0.0, 0.0],
                    }
                } else {
                    VertexRaw {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [m.mesh.texcoords.get(i * 2).map(|x| *x).unwrap_or(0.0), 1.0 - m.mesh.texcoords.get(i * 2 + 1).map(|x| *x).unwrap_or(0.0)],
                        normal: [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ],
                    }
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

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    transform: Transform,
    transform_buffer: wgpu::Buffer,
    transform_bind_group: wgpu::BindGroup,
    material: usize,
    color: ColorRaw,
    color_bind_group: wgpu::BindGroup,
    color_buffer: wgpu::Buffer,
}

impl Mesh {
    pub fn new(
        device: &wgpu::Device,
        vertices: Vec<VertexRaw>,
        indices: Vec<u32>,
        transform: Transform,
        material: usize,
        color: ColorRaw,
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
            contents: bytemuck::cast_slice(transform.to_raw().get_model()),
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
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(color.get_model()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let color_bind_group_layout = ColorRaw::color_bind_group_layout(device);
        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
            label: Some("per_mesh_transform_bind_group"),
        });
        let index_count = indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            index_count,
            transform,
            transform_buffer,
            transform_bind_group,
            material,
            color,
            color_buffer,
            color_bind_group,
        }
    }
    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, self.get_transform_bind_group(), &[]);
        render_pass.set_bind_group(2, &self.color_bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.get_vertex_buffer().slice(..));
        render_pass.set_index_buffer(self.get_index_buffer().slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.get_num_indices(), 0, 0..1);
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
    pub fn get_transform(&self) -> &Transform {
        &self.transform
    }
    pub fn get_transform_buffer(&self) -> &wgpu::Buffer {
        &self.transform_buffer
    }
    pub fn get_transform_bind_group(&self) -> &wgpu::BindGroup {
        &self.transform_bind_group
    }
    pub fn get_material(&self) -> usize {
        self.material
    }

    fn pipeline_layout<'a>(
        device: &Device,
        bind_group_layouts: &'a [&'a BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        let mut bind_group_layouts = bind_group_layouts.to_vec();
        let texture_bind_group = ColorRaw::color_bind_group_layout(&device);
        bind_group_layouts.push(&texture_bind_group);
        let bind_group_layouts = bind_group_layouts.as_slice();
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        })
    }
    pub fn pipeline(
        device: &Device,
        bind_group_layouts: &[&BindGroupLayout],
        color_target_state: ColorTargetState,
    ) -> RenderPipeline {
        let pipeline_layout = Self::pipeline_layout(device, bind_group_layouts);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("model_shader"),
            source: include_wgsl!("../../shader/mesh_shader.wgsl").source,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexRaw::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(color_target_state)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Line,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }
}
