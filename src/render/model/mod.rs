use crate::render::animation::Animation;
use crate::render::buffers::texture::texture_raw::TextureRaw;
use crate::render::buffers::transform::Transform;
use crate::render::buffers::transform::transform_raw::TransformRaw;
use crate::render::intersection::{Triangle, moller_trumbore_intersection};
use crate::render::model::material::Material;
use crate::render::model::mesh::{Mesh, MeshBuilder};
use anyhow::Result;
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use eframe::wgpu::{Device, Queue};
use glam::{Mat4, Vec3};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

mod material;
pub mod mesh;

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

    pub fn ray_intersection(&self, origin: Vec3, direction: Vec3) -> Option<Vec3> {
        let transform_raw = self.get_transform().to_raw();
        let model_matrix = transform_raw.get_model();
        let model_matrix = Mat4::from_cols_array_2d(model_matrix);
        let model_matrix_inv = model_matrix.inverse();

        // Transform ray to model space
        let model_origin = model_matrix_inv * origin.extend(1.0);
        let model_origin = Vec3::new(model_origin.x, model_origin.y, model_origin.z);

        let model_direction = model_matrix_inv * direction.extend(0.0);
        let model_direction =
            Vec3::new(model_direction.x, model_direction.y, model_direction.z).normalize();

        let mut closest_intersection: Option<Vec3> = None;
        let mut closest_distance = f32::INFINITY;

        for mesh in self.get_meshes() {
            let vertices = mesh.get_vertices();
            let indices = mesh.get_indices();

            for triangle_index in 0..(indices.len() / 3) {
                if let Some(triangle) = Triangle::from_vertices(vertices, indices, triangle_index)
                    && let Some(intersection) = moller_trumbore_intersection(
                        model_origin,
                        model_direction,
                        triangle.clone(),
                    )
                {
                    let distance = intersection.distance(model_origin);
                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_intersection = Some(intersection);
                    }
                }
            }
        }

        // Transform intersection back to world space
        closest_intersection.map(|intersection| {
            let world_intersection = model_matrix * intersection.extend(1.0);
            Vec3::new(
                world_intersection.x,
                world_intersection.y,
                world_intersection.z,
            )
        })
    }

    pub fn clone_untextured(&self, device: &Device, queue: &Queue) -> Self {
        let new_meshes = self
            .get_meshes()
            .iter()
            .map(|mesh| mesh.with_material(device, 0))
            .collect::<Vec<_>>();
        let diffuse_texture =
            TextureRaw::from_color(device, queue, (1.0, 0.0, 0.0), "color_texture").unwrap();
        let diffuse_bind_group = diffuse_texture.diffuse_bind_group(device);
        Self::new(
            device,
            new_meshes,
            vec![Material::new(&diffuse_texture, diffuse_bind_group)],
            self.get_transform(),
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
            materials.push(Material::new(&diffuse_texture, diffuse_bind_group));
        }
        let meshes = models
            .into_iter()
            .map(|m| MeshBuilder::from(m).build(device))
            .collect::<Vec<_>>();
        Ok(Self::new(device, meshes, materials, transform))
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, self.get_transform_bind_group(), &[]);
        for mesh in self.get_meshes() {
            render_pass.set_bind_group(
                2,
                self.get_materials()[mesh.get_material()].get_diffuse_bind_group(),
                &[],
            );

            render_pass.set_vertex_buffer(0, mesh.get_vertex_buffer().slice(..));
            render_pass
                .set_index_buffer(mesh.get_index_buffer().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.get_num_indices(), 0, 0..1);
        }
    }

    // Getter methods
    pub fn get_meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    pub fn get_materials(&self) -> &[Material] {
        &self.materials
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
