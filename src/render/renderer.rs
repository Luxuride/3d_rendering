use eframe::egui_wgpu::RenderState;
use eframe::wgpu::util::DeviceExt;
use eframe::{egui, egui_wgpu, wgpu};
use glam::Quat;
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
    wgpu_render_state: RenderState,
    selected_pipeline: SelectedPipeline,

    // Pipelines
    wireframe_pipeline: RenderPipeline,
    model_pipeline: RenderPipeline,
    outline_pipeline: RenderPipeline,

    // Camera buffer
    camera_bind_group: BindGroup,
    camera_uniform_buffer: Buffer,

    // Instances
    outline: Option<Model>,
    axis: [Model; 3],
    models: Vec<Model>,
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
                Transform::default().rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
            ),
            z_axis_mesh_builder().build(device).to_model(
                device,
                &wgpu_render_state.queue,
                (0.0, 0.0, 1.0),
                Transform::default().rotation(Quat::from_rotation_y((-90.0_f32).to_radians())),
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

    // Getter methods
    pub fn get_wgpu_render_state(&self) -> &RenderState {
        &self.wgpu_render_state
    }

    pub fn get_selected_pipeline(&self) -> SelectedPipeline {
        self.selected_pipeline
    }

    pub fn get_wireframe_pipeline(&self) -> &RenderPipeline {
        &self.wireframe_pipeline
    }

    pub fn get_model_pipeline(&self) -> &RenderPipeline {
        &self.model_pipeline
    }

    pub fn get_outline_pipeline(&self) -> &RenderPipeline {
        &self.outline_pipeline
    }

    pub fn get_camera_bind_group(&self) -> &BindGroup {
        &self.camera_bind_group
    }

    pub fn get_camera_uniform_buffer(&self) -> &Buffer {
        &self.camera_uniform_buffer
    }

    pub fn get_outline(&self) -> &Option<Model> {
        &self.outline
    }

    pub fn get_axis(&self) -> &[Model; 3] {
        &self.axis
    }

    pub fn get_models(&self) -> &Vec<Model> {
        &self.models
    }

    pub fn get_models_mut(&mut self) -> &mut Vec<Model> {
        &mut self.models
    }

    pub fn get_selected_pipeline_mut(&mut self) -> &mut SelectedPipeline {
        &mut self.selected_pipeline
    }

    pub fn set_outline(&mut self, outline: Option<Model>) {
        self.outline = outline;
    }

    pub fn update_selected_model(&mut self, selected_model: Option<usize>) {
        if let Some(model_idx) = selected_model {
            if model_idx < self.get_models().len() {
                let selected_model = &self.get_models()[model_idx];
                let device = &self.get_wgpu_render_state().device;
                let queue = &self.get_wgpu_render_state().queue;
                let outline_model = selected_model.clone_untextured(device, queue);
                self.set_outline(Some(outline_model));
            }
        } else {
            self.set_outline(None);
        }
    }

    pub fn prepare(&self, _device: &Device, queue: &wgpu::Queue, camera_uniform: CameraRaw) {
        queue.write_buffer(
            self.get_camera_uniform_buffer(),
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
        for model in self.get_models().iter() {
            queue.write_buffer(
                model.get_transform_buffer(),
                0,
                bytemuck::cast_slice(&[model.get_transform().to_raw()]),
            );
        }
        if let Some(model) = self.get_outline() {
            queue.write_buffer(
                model.get_transform_buffer(),
                0,
                bytemuck::cast_slice(&[model.get_transform().to_raw()]),
            );
        }
        for axis in self.get_axis().iter() {
            queue.write_buffer(
                axis.get_transform_buffer(),
                0,
                bytemuck::cast_slice(&[axis.get_transform().to_raw()]),
            );
        }
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_bind_group(0, self.get_camera_bind_group(), &[]);

        // Render outline
        if let Some(outline) = self.get_outline() {
            render_pass.set_pipeline(self.get_outline_pipeline());
            outline.draw(render_pass);
        }

        // Render models
        render_pass.set_pipeline(match self.get_selected_pipeline() {
            SelectedPipeline::Wireframe => self.get_wireframe_pipeline(),
            SelectedPipeline::Textured => self.get_model_pipeline(),
        });

        for model in self.get_models().iter() {
            model.draw(render_pass);
        }

        // Render axis
        render_pass.set_pipeline(self.get_wireframe_pipeline());
        for axis in self.get_axis().iter() {
            axis.draw(render_pass);
        }
    }

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

    fn camera_bind_group(device: &Device, layout: &BindGroupLayout, buffer: &Buffer) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        })
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

    // Getter methods
    pub fn get_camera_uniform(&self) -> &CameraRaw {
        &self.camera_uniform
    }

    pub fn get_renderer(&self) -> &Arc<RwLock<RendererRenderResources>> {
        &self.renderer
    }
}

impl egui_wgpu::CallbackTrait for RendererCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let renderer = &mut self.get_renderer().write().unwrap();
        renderer.prepare(device, queue, *self.get_camera_uniform());
        vec![]
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let renderer = self.get_renderer().read().unwrap();
        renderer.paint(render_pass);
    }
}
