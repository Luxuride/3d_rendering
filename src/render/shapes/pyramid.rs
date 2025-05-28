use eframe::{egui, egui_wgpu, wgpu};
use eframe::egui_wgpu::RenderState;
use wgpu::Device;

use cgmath::{Matrix4};
use crate::render::renderer::RendererRenderResources;
const PYRAMID_NUM_VERTICES: u32 = 16;

pub struct PyramidRenderResources {
    renderer_render_resources: RendererRenderResources,
}

impl PyramidRenderResources {
    pub fn new(
        device: &Device,
        wgpu_render_state: &RenderState,
    ) -> Self {
        let renderer_render_resources = RendererRenderResources::new(device, wgpu_render_state, PYRAMID_NUM_VERTICES, include_str!("shaders/pyramid.wgsl"));
        Self {
            renderer_render_resources
        }
    }
}

pub struct PyramidCallback {
    view_projection_matrix: Matrix4<f32>,
}

impl PyramidCallback {
    pub fn new(view_projection_matrix: Matrix4<f32>) -> Self {
        Self { view_projection_matrix }
    }
}

impl egui_wgpu::CallbackTrait for PyramidCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &PyramidRenderResources = resources.get().unwrap();
        resources.renderer_render_resources.prepare(device, queue, self.view_projection_matrix);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &PyramidRenderResources = resources.get().unwrap();
        resources.renderer_render_resources.paint(render_pass);
    }
}