use crate::render::renderer::RendererRenderResources;
use eframe::egui_wgpu::RenderState;
use eframe::wgpu::{include_wgsl, Device};
use std::sync::{Arc, RwLock};

const PYRAMID_NUM_VERTICES: u32 = 16;
pub fn create_pyramid(
    device: &Device,
    wgpu_render_state: &RenderState,
) -> Arc<RwLock<RendererRenderResources>> {
    let renderer = Arc::new(RwLock::new(RendererRenderResources::new(
        device,
        wgpu_render_state,
        PYRAMID_NUM_VERTICES,
        include_wgsl!("./shaders/pyramid.wgsl").source,
    )));
    renderer
}
