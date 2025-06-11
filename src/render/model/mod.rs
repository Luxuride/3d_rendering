use crate::render::animation::Animation;
use crate::render::buffers::texture::texture_raw::TextureRaw;
use crate::render::buffers::transform::transform_raw::TransformRaw;
use crate::render::buffers::transform::Transform;
use crate::render::model;
use crate::render::model::material::Material;
use crate::render::model::mesh::{Mesh, MeshBuilder};
use anyhow::Result;
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use eframe::wgpu::{Device, Queue};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

mod material;
pub mod mesh;
pub mod outline;

pub struct Model {
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
    transform: Transform,
    transform_buffer: wgpu::Buffer,
    transform_bind_group: wgpu::BindGroup,
    animation: Option<Box<dyn Animation + Send + Sync>>,
}

impl Model {
    pub fn new(
        device: &Device,
        meshes: Vec<Mesh>,
        materials: Vec<Material>,
        transform: Transform,
    ) -> Self {
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
        Self {
            meshes,
            materials,
            transform,
            transform_buffer,
            transform_bind_group,
            animation: None,
        }
    }
    pub fn clone_untextured(&self, device: &Device, queue: &Queue) -> Self {
        let mut new_meshes = self.meshes.clone();
        for mesh in new_meshes.iter_mut() {
            mesh.material = 0;
        }
        let diffuse_texture =
            TextureRaw::from_color(device, queue, (1.0, 0.0, 0.0), "color_texture").unwrap();
        let diffuse_bind_group = diffuse_texture.diffuse_bind_group(device);
        Self::new(
            device,
            new_meshes,
            vec![Material {
                diffuse_texture,
                diffuse_bind_group,
            }],
            self.transform,
        )
    }
    pub fn load_model(
        file_path: &Path,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        transform: Transform,
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
            let diffuse_texture = if let Some(diffuse_texture) = m.diffuse_texture {
                TextureRaw::load_texture(
                    &dir.join(&diffuse_texture),
                    &diffuse_texture,
                    device,
                    queue,
                )?
            } else if let Some(diffuse) = m.diffuse {
                TextureRaw::from_color(device, queue, diffuse.into(), &m.name)?
            } else {
                continue;
            };
            let diffuse_bind_group = diffuse_texture.diffuse_bind_group(device);
            materials.push(model::Material {
                diffuse_texture,
                diffuse_bind_group,
            });
        }
        let meshes = models
            .into_iter()
            .map(|m| MeshBuilder::from(m).build(device))
            .collect::<Vec<_>>();
        Ok(Self::new(device, meshes, materials, transform))
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, self.get_transform_bind_group(), &[]);
        for mesh in self.meshes.iter() {
            render_pass.set_bind_group(
                2,
                &self.materials[mesh.get_material()].diffuse_bind_group,
                &[],
            );

            render_pass.set_vertex_buffer(0, mesh.get_vertex_buffer().slice(..));
            render_pass
                .set_index_buffer(mesh.get_index_buffer().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.get_num_indices(), 0, 0..1);
        }
    }
    pub fn get_transform(&self) -> Transform {
        match self.animation.as_ref() {
            Some(animation) => animation.get_animation_transform(&self.transform),
            None => self.transform,
        }
    }
    pub fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    pub fn get_transform_buffer(&self) -> &wgpu::Buffer {
        &self.transform_buffer
    }
    pub fn get_transform_bind_group(&self) -> &wgpu::BindGroup {
        &self.transform_bind_group
    }
    pub fn add_animation_time(&mut self, delta_time: Duration) {
        if let Some(animation) = self.animation.as_mut() {
            animation.update_time(delta_time);
        }
    }
}
