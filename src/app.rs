use crate::render::buffers::camera::{Camera, CameraBuilder, CameraMovement};
use crate::render::model::mesh::Mesh;
use crate::render::renderer::{RendererCallback, RendererRenderResources};
use cgmath::Point3;
use eframe::{egui, egui_wgpu};
use std::sync::{Arc, RwLock};
use eframe::egui::Label;
use crate::render::model::Model;

pub struct Custom3d {
    camera: Camera,
    renderer: Arc<RwLock<RendererRenderResources>>,
}

impl Custom3d {
    pub fn new(cc: &eframe::CreationContext) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.clone()?;
        let camera = CameraBuilder::default()
            .position(Point3::new(0.0, 0.0, -5.0))
            .build();

        let renderer = Arc::new(RwLock::new(RendererRenderResources::new(
            wgpu_render_state,
            &camera,
        )));
        Some(Self { camera, renderer })
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
            if i.key_down(egui::Key::Space) {
                // Move up
                self.camera.process_keyboard_input(CameraMovement::Up);
            }
            if i.key_down(egui::Key::C) {
                // Move down
                self.camera.process_keyboard_input(CameraMovement::Down);
            }
            if i.key_down(egui::Key::Q) {
                self.camera.process_keyboard_input(CameraMovement::FovUp);
            }
            if i.key_down(egui::Key::E) {
                self.camera.process_keyboard_input(CameraMovement::FovDown);
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new("Camera:"));
                ui.add(Label::new(format!("X: {:.2}", self.camera.get_position().x)));
                ui.add(Label::new(format!("Y: {:.2}", self.camera.get_position().y)));
                ui.add(Label::new(format!("Z: {:.2}", self.camera.get_position().z)));
                ui.add(Label::new(format!("FOV: {:.2}", self.camera.get_fov())));
            });
            let button = ui.button("Add model");
            if button.clicked() {
                if let Some(file) = rfd::FileDialog::new().pick_file() {
                    let renderer = &mut self.renderer.write().unwrap();
                    let model = 
                        Model::load_model(&file, &renderer.wgpu_render_state.device, &renderer.wgpu_render_state.queue).unwrap();
                    renderer.models.push(model);
                }
            }
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
        });
        ctx.request_repaint();
    }
}

impl Custom3d {
    pub fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::new(ui.available_width(), ui.available_height()),
            egui::Sense::drag(),
        );
        self.camera
            .update_aspect_ratio(rect.width() / rect.height());
        self.camera
            .process_mouse_movement(response.drag_motion().x, response.drag_motion().y);
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            RendererCallback::new(self.camera.get_camera_uniform(), self.renderer.clone()),
        ));
    }
}
