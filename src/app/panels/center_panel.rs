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
                    self.get_camera().get_position().x
                )));
                ui.add(Label::new(format!(
                    "Y: {:.2}",
                    self.get_camera().get_position().y
                )));
                ui.add(Label::new(format!(
                    "Z: {:.2}",
                    self.get_camera().get_position().z
                )));
                ui.add(Label::new(format!(
                    "FOV: {:.2}",
                    self.get_camera().get_fov()
                )));
            });
            ui.horizontal(|ui| {
                let mut renderer = self.get_renderer().write().unwrap();
                ui.radio_value(
                    renderer.get_selected_pipeline_mut(),
                    SelectedPipeline::Wireframe,
                    "Wireframe",
                );
                ui.radio_value(
                    renderer.get_selected_pipeline_mut(),
                    SelectedPipeline::Textured,
                    "Textured",
                );
            });
            let button = ui.button("Add model");
            ui.add(Label::new(format!(
                "Loading {} models",
                self.get_loading()
                    .load(std::sync::atomic::Ordering::Relaxed)
            )));
            let loading = self.get_loading().clone();
            if button.clicked() {
                let renderer = self.get_renderer().clone();
                std::thread::spawn(move || {
                    if let Some(file) = rfd::FileDialog::new()
                        .add_filter("obj", &["obj"])
                        .pick_file()
                    {
                        loading.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let model = {
                            let wgpu_render_state =
                                &renderer.read().unwrap().get_wgpu_render_state().clone();
                            Model::load_model(
                                &file,
                                &wgpu_render_state.device,
                                &wgpu_render_state.queue,
                                Transform::default(),
                            )
                            .ok()
                        };
                        if let Some(model) = model {
                            let mut renderer = renderer.write().unwrap();
                            renderer.get_models_mut().push(model);
                        } else {
                            eprint!("Error loading model");
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
