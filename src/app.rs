use cgmath::Point3;
use eframe::{
    egui,
    egui_wgpu::self,
};
use crate::camera::{Camera, CameraMovement};
use crate::render::shapes::cube::{CubeCallback, CubeRenderResources};
use crate::render::shapes::pyramid::{PyramidCallback, PyramidRenderResources};

pub struct Custom3d {
    camera: Camera,
}

impl Custom3d {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(CubeRenderResources::new(
                device,
                wgpu_render_state,

            ));
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(PyramidRenderResources::new(
                device,
                wgpu_render_state,
            ));

        Some(Self {
            camera: Camera::new(
                Point3::new(0.0, 0.0, 0.0),
                -90.0, // Looking along negative Z initially
                0.0,
                45.0,
                0.1,
                100.0,
                1.0,
                0.1, // Pass the new speed parameter
            ),
        })
    }
}

impl eframe::App for Custom3d {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if i.key_down(egui::Key::W) {
                self.camera.process_keyboard_input(CameraMovement::Forward);
            }
            if i.key_down(egui::Key::S) {
                self.camera.process_keyboard_input(CameraMovement::Backward);
            }
            if i.key_down(egui::Key::A) {
                self.camera.process_keyboard_input(CameraMovement::Left);
            }
            if i.key_down(egui::Key::D) {
                self.camera.process_keyboard_input(CameraMovement::Right);
            }
            if i.key_down(egui::Key::Space) { // Move up
                self.camera.process_keyboard_input(CameraMovement::Up);
            }
            if i.key_down(egui::Key::C) { // Move down
                self.camera.process_keyboard_input(CameraMovement::Down);
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
        });
    }
}

impl Custom3d {
    pub fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::new(ui.available_width(), ui.available_height()), egui::Sense::drag());
        self.camera.update_aspect_ratio(rect.width() / rect.height());
        self.camera.process_mouse_movement(response.drag_motion().x, response.drag_motion().y);
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            CubeCallback::new(self.camera.build_view_projection_matrix()),
        ));
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            PyramidCallback::new(self.camera.build_view_projection_matrix()),
        ));
    }
}