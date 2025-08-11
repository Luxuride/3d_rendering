use crate::app::Custom3d;
use eframe::egui::ViewportBuilder;
use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use eframe::wgpu;
use eframe::wgpu::Features;
use std::sync::Arc;

mod app;
mod networking;
mod render;

fn main() -> eframe::Result {
    // Work around EGL unwrap panic by forcing Vulkan and X11 when available
    unsafe {
        // Set both env vars that wgpu recognizes across versions
        std::env::set_var("WGPU_BACKEND", "vulkan");
        std::env::set_var("WGPU_BACKENDS", "vulkan");
        // Comment this out if you prefer native Wayland and Vulkan works there.
        std::env::set_var("WINIT_UNIX_BACKEND", "x11");
    }
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([350.0, 380.0]),
        multisampling: *render::pipeline::SAMPLE_COUNT,
        renderer: eframe::Renderer::Wgpu,
        depth_buffer: 32,
        wgpu_options: WgpuConfiguration {
            wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
                device_descriptor: Arc::new(|_adapter| wgpu::DeviceDescriptor {
                    required_features: Features::POLYGON_MODE_LINE | Default::default(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        "Custom 3D painting in eframe using glow",
        options,
        Box::new(|cc| Ok(Box::new(Custom3d::new(cc).unwrap()))),
    )
}
