use eframe::{egui, glow};
use std::sync::Arc;
use eframe::egui_glow::Painter;
use glow::{NativeProgram, NativeVertexArray};
use crate::renderer::SimpleOpenGLApp;

// Application that integrates the renderer with egui
pub struct OpenGLApp {
    renderer: SimpleOpenGLApp,
}

impl OpenGLApp {
    pub fn new(gl_context: Arc<glow::Context>) -> Self {
        Self {
            renderer: SimpleOpenGLApp::new(gl_context),
        }
    }

    unsafe fn initialize_gl(painter: &Painter, program: NativeProgram, rotation: f32, vertex_array: NativeVertexArray) {
        let gl = painter.gl();
        
        use glow::HasContext as _;
        
        // Set up OpenGL state for rendering
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LESS);
        gl.clear_color(0.1, 0.1, 0.1, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

        // Use the program
        gl.use_program(Some(program));

        // Set the rotation uniform
        let rotation_loc = gl.get_uniform_location(program, "u_rotation").unwrap();
        gl.uniform_1_f32(Some(&rotation_loc), rotation);

        // Bind and draw the triangle
        gl.bind_vertex_array(Some(vertex_array));
        gl.draw_arrays(glow::TRIANGLES, 0, 3);

        // Clean up
        gl.bind_vertex_array(None);
        gl.use_program(None);
        gl.disable(glow::DEPTH_TEST);
    }
}

impl eframe::App for OpenGLApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update rotation for animation
        self.renderer.update_rotation();
        
        // Create a simple egui UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("OpenGL Renderer");
            ui.label("A simple rotating triangle rendered with OpenGL");
            
            // Request a custom rendering callback
            let (rect, _) = ui.allocate_exact_size(egui::vec2(300.0, 300.0), egui::Sense::drag());
            
            // Get values we need to capture in the callback
            let rotation = self.renderer.rotation();
            let program = self.renderer.program();
            let vertex_array = self.renderer.vertex_array();
            
            // Create a callback that only captures what it needs
            let callback = egui::PaintCallback {
                rect,
                callback: Arc::new(eframe::egui_glow::CallbackFn::new(
                    move |_info, painter| {
                        unsafe {
                            Self::initialize_gl(painter, program, rotation, vertex_array);
                        }
                    }
                )),
            };
            
            ui.painter().add(callback);
        });
        
        // Request continuous repainting for animation
        ctx.request_repaint();
    }
    
    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        // Clean up OpenGL resources
        if let Some(gl) = gl {
            self.renderer.cleanup(gl);
        }
    }
}

// Fallback app in case no GL context is available
#[derive(Default)]
pub struct FallbackApp {}

impl eframe::App for FallbackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("OpenGL Not Available");
            ui.label("Could not initialize OpenGL. Please check your drivers and try again.");
        });
    }
} 