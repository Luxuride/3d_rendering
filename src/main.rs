use crate::app::Custom3d;
use eframe::egui::ViewportBuilder;
use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use eframe::wgpu;
use eframe::wgpu::Features;
use std::sync::Arc;

pub mod app;
pub mod render;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([350.0, 380.0]),
        multisampling: 1,
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
