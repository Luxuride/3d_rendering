use crate::render::renderer::RendererRenderResources;
use eframe::egui_wgpu::RenderState;
use eframe::wgpu::{include_wgsl, Device};
use std::sync::{Arc, RwLock};

const CUBE_NUM_VERTICES: u32 = 36;
pub fn create_cube(
    device: &Device,
    wgpu_render_state: &RenderState,
) -> Arc<RwLock<RendererRenderResources>> {
    let renderer = Arc::new(RwLock::new(RendererRenderResources::new(
        device,
        wgpu_render_state,
        CUBE_NUM_VERTICES,
        include_wgsl!("./shaders/cube.wgsl").source,
    )));
    renderer
}
