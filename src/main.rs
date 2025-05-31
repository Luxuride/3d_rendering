use crate::app::Custom3d;
use eframe::egui::ViewportBuilder;

pub mod app;
pub mod camera;
pub mod data;
pub mod render;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([350.0, 380.0]),
        multisampling: 1,
        renderer: eframe::Renderer::Wgpu,
        depth_buffer: 32,
        ..Default::default()
    };
    eframe::run_native(
        "Custom 3D painting in eframe using glow",
        options,
        Box::new(|cc| Ok(Box::new(Custom3d::new(cc).unwrap()))),
    )
}
