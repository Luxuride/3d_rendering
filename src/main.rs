// This is a minimal test that only uses winit to create a window
// without any OpenGL/egui to isolate the windowing system issue
mod app;
mod renderer;

use std::sync::Arc;
use eframe::egui;

fn main() {
    // Setup logging for better error messages
    env_logger::init();
    
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        hardware_acceleration: eframe::HardwareAcceleration::Required,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("OpenGL Triangle Demo"),
        ..Default::default()
    };

    match eframe::run_native(
        "OpenGL Triangle Demo",
        options,
        Box::new(|cc| {
            if let Some(gl_ctx) = cc.gl.as_ref() {
                // We have OpenGL context, create the app with renderer
                Ok(Box::new(app::OpenGLApp::new(Arc::clone(gl_ctx))))
            } else {
                // No OpenGL context, show a fallback app with error message
                Ok(Box::new(app::FallbackApp::default()))
            }
        }),
    ) {
        Ok(_) => println!("Application closed normally"),
        Err(e) => eprintln!("Application error: {}", e),
    }
}
