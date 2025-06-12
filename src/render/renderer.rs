use cgmath::{Deg, Quaternion, Rotation3};
use eframe::egui_wgpu::RenderState;
use eframe::wgpu::util::DeviceExt;
use eframe::{egui, egui_wgpu, wgpu};
use std::sync::{Arc, RwLock};
use wgpu::Device;

use crate::render::buffers::camera::Camera;
use crate::render::buffers::camera::camera_raw::CameraRaw;
use crate::render::buffers::transform::Transform;
use crate::render::buffers::transform::transform_raw::TransformRaw;
use crate::render::model::Model;
use crate::render::model::mesh::axis::{
    x_axis_mesh_builder, y_axis_mesh_builder, z_axis_mesh_builder,
};
use crate::render::model::mesh::cube::cube_mesh_builder;
use crate::render::pipeline::{
    SelectedPipeline, model_pipeline, outline_pipeline, wireframe_pipeline,
};
use eframe::wgpu::{BindGroup, BindGroupEntry, BindGroupLayout, Buffer, RenderPipeline};

pub struct RendererRenderResources {
    pub wgpu_render_state: RenderState,
    pub selected_pipeline: SelectedPipeline,

    // Pipelines
    wireframe_pipeline: RenderPipeline,
    model_pipeline: RenderPipeline,
    outline_pipeline: RenderPipeline,

    // Camera buffer
    camera_bind_group: BindGroup,
    camera_uniform_buffer: Buffer,

    // Instances
    pub outline: Option<Model>,
    pub axis: [Model; 3],
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

        let wireframe_pipeline = wireframe_pipeline(
            device,
            &[&camera_bind_group_layout, &transform_bind_group_layout],
            wgpu_render_state.target_format.into(),
        );
        let model_pipeline = model_pipeline(
            device,
            &[&camera_bind_group_layout, &transform_bind_group_layout],
            wgpu_render_state.target_format.into(),
        );
        let outline_pipeline = outline_pipeline(
            device,
            &[&camera_bind_group_layout, &transform_bind_group_layout],
            wgpu_render_state.target_format.into(),
        );

        let mut models = vec![];
        let cube = cube_mesh_builder().build(device).to_model(
            device,
            &wgpu_render_state.queue,
            (1.0, 1.0, 0.0),
            Transform::default(),
        );
        models.push(cube);

        let axis = [
            x_axis_mesh_builder().build(device).to_model(
                device,
                &wgpu_render_state.queue,
                (1.0, 0.0, 0.0),
                Transform::default(),
            ),
            y_axis_mesh_builder().build(device).to_model(
                device,
                &wgpu_render_state.queue,
                (0.0, 1.0, 0.0),
                Transform::default().rotation(Quaternion::from_angle_z(Deg(90.0))),
            ),
            z_axis_mesh_builder().build(device).to_model(
                device,
                &wgpu_render_state.queue,
                (0.0, 0.0, 1.0),
                Transform::default().rotation(Quaternion::from_angle_y(Deg(-90.0))),
            ),
        ];

        Self {
            wireframe_pipeline,
            model_pipeline,
            outline_pipeline,
            camera_bind_group,
            camera_uniform_buffer,
            models,
            axis,
            outline: None,
            wgpu_render_state,
            selected_pipeline: SelectedPipeline::Wireframe,
        }
    }

    pub fn prepare(&self, _device: &Device, queue: &wgpu::Queue, camera_uniform: CameraRaw) {
        queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
        for model in self.models.iter() {
            queue.write_buffer(
                model.get_transform_buffer(),
                0,
                bytemuck::cast_slice(&[model.get_transform().to_raw()]),
            );
        }
        if let Some(model) = &self.outline {
            queue.write_buffer(
                model.get_transform_buffer(),
                0,
                bytemuck::cast_slice(&[model.get_transform().to_raw()]),
            );
        }
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.wireframe_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        for axis in self.axis.iter() {
            axis.draw(render_pass);
        }
        match self.selected_pipeline {
            SelectedPipeline::Wireframe => render_pass.set_pipeline(&self.wireframe_pipeline),
            SelectedPipeline::Textured => render_pass.set_pipeline(&self.model_pipeline),
        }
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        for model in self.models.iter() {
            model.draw(render_pass);
        }
        if let Some(model) = &self.outline {
            render_pass.set_pipeline(&self.outline_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
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
