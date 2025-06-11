use crate::app::Custom3d;
use crate::render::buffers::transform::Transform;
use crate::render::model::Model;
use crate::render::pipeline::SelectedPipeline;
use eframe::egui;
use eframe::egui::Label;

impl Custom3d {
    pub fn center_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new("Camera:"));
                ui.add(Label::new(format!(
                    "X: {:.2}",
                    self.camera.get_position().x
                )));
                ui.add(Label::new(format!(
                    "Y: {:.2}",
                    self.camera.get_position().y
                )));
                ui.add(Label::new(format!(
                    "Z: {:.2}",
                    self.camera.get_position().z
                )));
                ui.add(Label::new(format!("FOV: {:.2}", self.camera.get_fov())));
            });
            ui.horizontal(|ui| {
                let renderer = &mut self.renderer.write().unwrap();
                ui.radio_value(
                    &mut renderer.selected_pipeline,
                    SelectedPipeline::Wireframe,
                    "Wireframe",
                );
                ui.radio_value(
                    &mut renderer.selected_pipeline,
                    SelectedPipeline::Textured,
                    "Textured",
                );
            });
            let button = ui.button("Add model");
            ui.add(Label::new(format!(
                "Loading {} models",
                self.loading.load(std::sync::atomic::Ordering::Relaxed)
            )));
            let loading = self.loading.clone();
            if button.clicked() {
                let renderer = self.renderer.clone();
                std::thread::spawn(move || {
                    if let Some(file) = rfd::FileDialog::new()
                        .add_filter("obj", &["obj"])
                        .pick_file()
                    {
                        loading.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let model = {
                            let wgpu_render_state =
                                &renderer.read().unwrap().wgpu_render_state.clone();
                            Model::load_model(
                                &file,
                                &wgpu_render_state.device,
                                &wgpu_render_state.queue,
                                Transform::default(),
                            )
                            .ok()
                        };
                        if let Some(model) = model {
                            let renderer = &mut renderer.write().unwrap();
                            renderer.models.push(model);
                        } else {
                            println!("Error loading model");
                        }
                        loading.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    }
                });
            }
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
        });
    }
}
