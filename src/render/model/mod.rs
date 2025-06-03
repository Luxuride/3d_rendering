use crate::render::buffers::vertex::vertex_raw::VertexRaw;
use crate::render::model;
use crate::render::model::material::texture::Texture;
use crate::render::model::material::Material;
use crate::render::model::mesh::{Mesh, MeshBuilder};
use anyhow::Result;
use eframe::wgpu;
use eframe::wgpu::{include_wgsl, BindGroupLayout, ColorTargetState, Device, RenderPipeline};
use std::fs::File;
use std::io::BufReader;
use std::ops::Range;
use std::path::Path;

mod material;
pub mod mesh;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn load_model(
        file_path: &Path,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self> {
        let dir = file_path.parent().unwrap();
        let obj = File::open(file_path)?;
        let mut obj_reader = BufReader::new(obj);
        let (models, obj_materials) = tobj::load_obj_buf(
            &mut obj_reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| {
                let mat = File::open(dir.join(p)).unwrap();
                tobj::load_mtl_buf(&mut BufReader::new(mat))
            },
        )?;
        let mut materials = Vec::new();
        for m in obj_materials? {
            let Some(diffuse_texture) = m.diffuse_texture else {
                continue;
            };
            let diffuse_texture = Texture::load_texture(
                &dir.join(&diffuse_texture),
                &diffuse_texture,
                device,
                queue,
            )?;
            let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &Texture::diffuse_bind_group_layout(&device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
                label: None,
            });
            materials.push(model::Material {
                name: m.name,
                diffuse_texture,
                diffuse_bind_group,
            });
        }
        let meshes = models
            .into_iter()
            .map(|m| MeshBuilder::from(m).build(device))
            .collect::<Vec<_>>();
        Ok(Self { meshes, materials })
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        for mesh in self.meshes.iter() {
            render_pass.set_bind_group(1, mesh.get_transform_bind_group(), &[]);
            render_pass.set_bind_group(
                2,
                &self
                    .materials[mesh.get_material()]
                    .diffuse_bind_group,
                &[],
            );

            render_pass.set_vertex_buffer(0, mesh.get_vertex_buffer().slice(..));
            render_pass
                .set_index_buffer(mesh.get_index_buffer().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.get_num_indices(), 0, 0..1);
        }
    }

    fn pipeline_layout<'a>(
        device: &Device,
        bind_group_layouts: &'a [&'a BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        let mut bind_group_layouts = bind_group_layouts.to_vec();
        let texture_bind_group = Texture::diffuse_bind_group_layout(&device);
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
            source: include_wgsl!("../shader/model_shader.wgsl").source,
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
                polygon_mode: wgpu::PolygonMode::Fill,
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
