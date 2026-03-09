use crate::render::animation::Animation;
use crate::render::buffers::texture::texture_raw::TextureRaw;
use crate::render::buffers::transform::Transform;
use crate::render::buffers::transform::transform_raw::TransformRaw;
use crate::render::intersection::{Triangle, moller_trumbore_intersection};
use crate::render::model::material::Material;
use crate::render::model::mesh::{Mesh, MeshBuilder};
use anyhow::{Result, bail};
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use eframe::wgpu::{Device, Queue};
use glam::{Mat4, Vec3};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;

mod material;
pub mod mesh;

pub struct NamedModel {
    pub name: String,
    pub model: Model,
}

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
                #[allow(clippy::collapsible_if)]
                if let Some(triangle) = Triangle::from_vertices(vertices, indices, triangle_index) {
                    if let Some(intersection) =
                        moller_trumbore_intersection(model_origin, model_direction, triangle)
                    {
                        let distance = intersection.distance(model_origin);
                        if distance < closest_distance {
                            closest_distance = distance;
                            closest_intersection = Some(intersection);
                        }
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
        let named_models = Self::load_named_models(file_path, device, queue, transform)?;
        let (meshes, materials) = named_models.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut meshes, mut materials), named_model| {
                let model = named_model.model;
                meshes.extend_from_slice(model.get_meshes());
                if materials.is_empty() {
                    materials.extend_from_slice(model.get_materials());
                }
                (meshes, materials)
            },
        );
        Ok(Self::new(device, meshes, materials, transform))
    }

    pub fn load_named_models(
        file_path: &Path,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        transform: Transform,
    ) -> Result<Vec<NamedModel>> {
        let mut obj = File::open(file_path)?;
        if Self::is_git_lfs_pointer(&mut obj)? {
            bail!(
                "{} is a Git LFS pointer, not an OBJ mesh. Fetch real assets with `git lfs pull` (or package the real file) and try again.",
                file_path.display()
            );
        }

        let (models, obj_materials) = tobj::load_obj(
            file_path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;
        let dir = file_path.parent().unwrap();
        let materials = Self::load_materials(dir, obj_materials?, device, queue)?;

        Ok(models
            .into_iter()
            .map(|obj_model| {
                let name = obj_model.name.clone();
                let mesh = MeshBuilder::from(obj_model).build(device);
                let model = Self::new(device, vec![mesh], materials.clone(), transform);
                NamedModel { name, model }
            })
            .collect())
    }

    fn is_git_lfs_pointer(file: &mut File) -> Result<bool> {
        let mut reader = BufReader::new(file);
        let mut first_line = String::new();
        reader.read_line(&mut first_line)?;
        reader.seek(SeekFrom::Start(0))?;
        Ok(first_line.trim() == "version https://git-lfs.github.com/spec/v1")
    }

    fn load_materials(
        dir: &Path,
        obj_materials: Vec<tobj::Material>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<Material>> {
        let mut materials = Vec::new();
        for m in obj_materials {
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
        Ok(materials)
    }

    pub fn instance_with_transform(&self, device: &wgpu::Device, transform: Transform) -> Self {
        Self::new(
            device,
            self.get_meshes().to_vec(),
            self.get_materials().to_vec(),
            transform,
        )
    }

    pub fn world_bounds(&self) -> Option<(Vec3, Vec3)> {
        let transform_raw = self.get_transform().to_raw();
        let model_matrix = Mat4::from_cols_array_2d(transform_raw.get_model());
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        let mut has_vertices = false;

        for mesh in self.get_meshes() {
            for vertex in mesh.get_vertices() {
                let local = Vec3::from_array(vertex.position());
                let world = model_matrix * local.extend(1.0);
                let world = Vec3::new(world.x, world.y, world.z);
                min = min.min(world);
                max = max.max(world);
                has_vertices = true;
            }
        }

        has_vertices.then_some((min, max))
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

    pub fn draw_depth_only(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, self.get_transform_bind_group(), &[]);
        for mesh in self.get_meshes() {
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
