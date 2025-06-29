use crate::app::Custom3d;
use eframe::egui;

impl Custom3d {
    pub fn right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                // Handle radio buttons first
                ui.radio_value(self.get_selected_model_mut(), None, "None".to_string());

                let model_count = {
                    let renderer = self.get_renderer().read().unwrap();
                    renderer.get_models().len()
                };

                for model_index in 0..model_count {
                    ui.radio_value(
                        self.get_selected_model_mut(),
                        Some(model_index),
                        format!("Model {model_index}"),
                    );
                }

                // Handle model manipulation
                let selected_model = self.get_selected_model();
                if let Some(model_index) = selected_model {
                    // Clone device/queue before mutable borrow
                    let (device, queue) = {
                        let renderer = self.get_renderer().read().unwrap();
                        (
                            renderer.get_wgpu_render_state().device.clone(),
                            renderer.get_wgpu_render_state().queue.clone(),
                        )
                    };
                    let mut renderer = self.get_renderer().write().unwrap();
                    if let Some(selected_model_mut) = renderer.get_models_mut().get_mut(model_index)
                    {
                        ui.add(
                            egui::DragValue::new(
                                &mut selected_model_mut.get_transform_mut().get_position_mut().x,
                            )
                            .speed(0.1),
                        );
                        ui.add(
                            egui::DragValue::new(
                                &mut selected_model_mut.get_transform_mut().get_position_mut().y,
                            )
                            .speed(0.1),
                        );
                        ui.add(
                            egui::DragValue::new(
                                &mut selected_model_mut.get_transform_mut().get_position_mut().z,
                            )
                            .speed(0.1),
                        );
                        ui.add(
                            egui::DragValue::new(
                                &mut selected_model_mut.get_transform_mut().get_scale_mut().x,
                            )
                            .speed(0.1),
                        );
                        ui.add(
                            egui::DragValue::new(
                                &mut selected_model_mut.get_transform_mut().get_scale_mut().y,
                            )
                            .speed(0.1),
                        );
                        ui.add(
                            egui::DragValue::new(
                                &mut selected_model_mut.get_transform_mut().get_scale_mut().z,
                            )
                            .speed(0.1),
                        );
                        let selected_model_clone =
                            selected_model_mut.clone_untextured(&device, &queue);
                        renderer.set_outline(Some(selected_model_clone));
                    }
                } else {
                    let mut renderer = self.get_renderer().write().unwrap();
                    renderer.set_outline(None);
                }
            });
    }
}
