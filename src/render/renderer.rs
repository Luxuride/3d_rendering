use cgmath::Vector3;
use eframe::egui_wgpu::RenderState;
use eframe::wgpu::util::DeviceExt;
use eframe::{egui, egui_wgpu, wgpu};
use std::sync::{Arc, RwLock};
use wgpu::Device;

use crate::render::buffers::camera::camera_raw::CameraRaw;
use crate::render::buffers::camera::Camera;
use crate::render::buffers::color::color_raw::ColorRaw;
use crate::render::buffers::transform::transform_raw::TransformRaw;
use crate::render::model::mesh::axis::{
    x_axis_mesh_builder, y_axis_mesh_builder, z_axis_mesh_builder,
};
use crate::render::model::mesh::cube::cube_mesh_builder;
use crate::render::model::mesh::Mesh;
use crate::render::model::Model;
use eframe::wgpu::{BindGroup, BindGroupEntry, BindGroupLayout, Buffer, RenderPipeline};

pub struct RendererRenderResources {
    pub wgpu_render_state: RenderState,

    mesh_pipeline: RenderPipeline,
    model_pipeline: RenderPipeline,
    // Camera buffer
    camera_bind_group: BindGroup,
    camera_uniform_buffer: Buffer,

    // Instance
    meshes: Vec<Mesh>,
    pub models: Vec<Model>,
}

impl RendererRenderResources {
    pub fn new(wgpu_render_state: RenderState, camera: &Camera) -> Self {
        let device = &wgpu_render_state.device;
        let camera_bind_group_layout = Self::camera_bind_group_layout(device);
        let camera_uniform_buffer =
            Self::camera_uniform_buffer(device, camera.get_camera_uniform());
        let camera_bind_group =
            Self::camera_bind_group(device, &camera_bind_group_layout, &camera_uniform_buffer);

        let camera_bind_group_layout = Self::camera_bind_group_layout(device);
        let transform_bind_group_layout = TransformRaw::transform_bind_group_layout(device);

        let mesh_pipeline = Mesh::pipeline(
            device,
            &[&camera_bind_group_layout, &transform_bind_group_layout],
            wgpu_render_state.target_format.into(),
        );
        let model_pipeline = Model::pipeline(
            device,
            &[&camera_bind_group_layout, &transform_bind_group_layout],
            wgpu_render_state.target_format.into(),
        );

        let mut meshes = vec![];
        let cube = cube_mesh_builder()
            .color(ColorRaw::new(1.0, 1.0, 0.0, 1.0))
            .position(Vector3::new(-2.0, 0.0, 0.0))
            .build(device);
        let (axis_x, axis_y, axis_z) = (
            x_axis_mesh_builder()
                .color(ColorRaw::new(1.0, 0.0, 0.0, 1.0))
                .build(device),
            y_axis_mesh_builder()
                .color(ColorRaw::new(0.0, 1.0, 0.0, 1.0))
                .build(device),
            z_axis_mesh_builder()
                .color(ColorRaw::new(0.0, 0.0, 1.0, 1.0))
                .build(device),
        );
        meshes.push(cube);
        meshes.push(axis_x);
        meshes.push(axis_y);
        meshes.push(axis_z);

        let models = vec![];

        Self {
            mesh_pipeline,
            model_pipeline,
            camera_bind_group,
            camera_uniform_buffer,
            meshes,
            models,
            wgpu_render_state,
        }
    }

    pub fn prepare(&self, _device: &Device, queue: &wgpu::Queue, camera_uniform: CameraRaw) {
        queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
        for mesh in self.meshes.iter() {
            queue.write_buffer(
                mesh.get_transform_buffer(),
                0,
                bytemuck::cast_slice(&[mesh.get_transform().to_raw()]),
            );
        }
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.mesh_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        for mesh in self.meshes.iter() {
            mesh.draw(render_pass);
        }
        render_pass.set_pipeline(&self.model_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        for model in self.models.iter() {
            model.draw(render_pass);
        }
    }
}

pub struct RendererCallback {
    camera_uniform: CameraRaw,
    renderer: Arc<RwLock<RendererRenderResources>>,
}

impl RendererCallback {
    pub fn new(camera_uniform: CameraRaw, renderer: Arc<RwLock<RendererRenderResources>>) -> Self {
        Self {
            camera_uniform,
            renderer,
        }
    }
}

impl egui_wgpu::CallbackTrait for RendererCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        self.renderer
            .read()
            .unwrap()
            .prepare(device, queue, self.camera_uniform);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _: &egui_wgpu::CallbackResources,
    ) {
        self.renderer.read().unwrap().paint(render_pass);
    }
}

impl RendererRenderResources {
    fn camera_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        })
    }
    fn camera_uniform_buffer(device: &Device, camera_uniform: CameraRaw) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
    fn camera_bind_group(
        device: &Device,
        camera_bind_group_layout: &BindGroupLayout,
        camera_uniform_buffer: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        })
    }
}
