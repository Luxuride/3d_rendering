use crate::app::Custom3d;
use eframe::egui;

impl Custom3d {
    pub fn right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                let renderer = &mut self.renderer.write().unwrap();
                ui.radio_value(&mut self.selected_model, None, "None".to_string());
                for (model_index, _) in renderer.models.iter().enumerate() {
                    ui.radio_value(
                        &mut self.selected_model,
                        Some(model_index),
                        format!("Model {}", model_index),
                    );
                }
                let selected_model = match self.selected_model {
                    Some(model_index) => renderer.models.get_mut(model_index),
                    None => None,
                };
                if let Some(selected_model) = selected_model {
                    ui.add(
                        egui::DragValue::new(&mut selected_model.get_transform_mut().position.x)
                            .speed(0.1),
                    );
                    ui.add(
                        egui::DragValue::new(&mut selected_model.get_transform_mut().position.y)
                            .speed(0.1),
                    );
                    ui.add(
                        egui::DragValue::new(&mut selected_model.get_transform_mut().position.z)
                            .speed(0.1),
                    );
                    ui.add(
                        egui::DragValue::new(&mut selected_model.get_transform_mut().scale.x)
                            .speed(0.1),
                    );
                    ui.add(
                        egui::DragValue::new(&mut selected_model.get_transform_mut().scale.y)
                            .speed(0.1),
                    );
                    ui.add(
                        egui::DragValue::new(&mut selected_model.get_transform_mut().scale.z)
                            .speed(0.1),
                    );
                }
                let device = &renderer.wgpu_render_state.device.clone();
                let queue = &renderer.wgpu_render_state.queue.clone();
                let selected_model = match self.selected_model {
                    Some(model_index) => renderer.models.get_mut(model_index),
                    None => None,
                }
                .map(|model| model.clone_untextured(device, queue));
                if let Some(selected_model) = selected_model {
                    renderer.outline = Some(selected_model);
                } else {
                    renderer.outline = None;
                }
            });
    }
}
