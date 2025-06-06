use crate::render::buffers::camera::{Camera, CameraBuilder, CameraMovement};
use crate::render::buffers::transform::Transform;
use crate::render::model::mesh::Mesh;
use crate::render::model::Model;
use crate::render::renderer::{RendererCallback, RendererRenderResources};
use cgmath::Point3;
use eframe::egui::Label;
use eframe::{egui, egui_wgpu};
use std::sync::{Arc, RwLock};

pub struct Custom3d {
    camera: Camera,
    renderer: Arc<RwLock<RendererRenderResources>>,
    selected_model: SelectedModel,
}

#[derive(PartialEq, Clone, Copy)]
enum SelectedModel {
    Wireframe(usize),
    Model(usize),
}

impl Custom3d {
    pub fn new(cc: &eframe::CreationContext) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.clone()?;
        let camera = CameraBuilder::default()
            .z_far(500.0)
            .position(Point3::new(0.0, 0.0, -5.0))
            .build();

        let renderer = Arc::new(RwLock::new(RendererRenderResources::new(
            wgpu_render_state,
            &camera,
        )));
        Some(Self {
            camera,
            renderer,
            selected_model: SelectedModel::Wireframe(0),
        })
    }
}

impl eframe::App for Custom3d {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if i.modifiers.shift {
                self.camera.move_speed = 1.0;
            }
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
            self.camera.move_speed = 0.1;
        });
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
            let button = ui.button("Add model");
            if button.clicked() {
                let renderer = self.renderer.clone();
                std::thread::spawn(move || {
                    if let Some(file) = rfd::FileDialog::new()
                        .add_filter("obj", &["obj"])
                        .pick_file()
                    {
                        let model = {
                            let wgpu_render_state =
                                &renderer.read().unwrap().wgpu_render_state.clone();
                            Model::load_model(
                                &file,
                                &wgpu_render_state.device,
                                &wgpu_render_state.queue,
                                Transform::default(),
                            )
                        };
                        if let Ok(model) = model {
                            let renderer = &mut renderer.write().unwrap();
                            renderer.models.push(model);
                        } else {
                            println!("Error loading model");
                        }
                    }
                });
            }
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
        });
        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            {
                let renderer = &self.renderer.read().unwrap();
                for (model_index, model) in renderer.wireframe_models.iter().enumerate() {
                    ui.radio_value(
                        &mut self.selected_model,
                        SelectedModel::Wireframe(model_index),
                        format!("Wireframe {}", model_index),
                    );
                }
                for (model_index, model) in renderer.models.iter().enumerate() {
                    ui.radio_value(
                        &mut self.selected_model,
                        SelectedModel::Model(model_index),
                        format!("Model {}", model_index),
                    );
                }
            }
            {
                let renderer = &mut self.renderer.write().unwrap();
                let selected_model = match self.selected_model {
                    SelectedModel::Wireframe(model_index) => {
                        renderer.wireframe_models.get_mut(model_index)
                    }
                    SelectedModel::Model(model_index) => renderer.models.get_mut(model_index),
                };
                if let Some(selected_model) = selected_model {
                    ui.add(egui::DragValue::new(
                        &mut selected_model.get_transform_mut().position.x,
                    ));
                    ui.add(egui::DragValue::new(
                        &mut selected_model.get_transform_mut().position.y,
                    ));
                    ui.add(egui::DragValue::new(
                        &mut selected_model.get_transform_mut().position.z,
                    ));
                    ui.add(egui::DragValue::new(
                        &mut selected_model.get_transform_mut().scale.x,
                    ));
                    ui.add(egui::DragValue::new(
                        &mut selected_model.get_transform_mut().scale.y,
                    ));
                    ui.add(egui::DragValue::new(
                        &mut selected_model.get_transform_mut().scale.z,
                    ));
                }
            };
            let renderer = &mut self.renderer.write().unwrap();
            let device = &renderer.wgpu_render_state.device.clone();
            let queue = &renderer.wgpu_render_state.queue.clone();
            let selected_model = match self.selected_model {
                SelectedModel::Wireframe(model_index) => renderer.wireframe_models.get(model_index),
                SelectedModel::Model(model_index) => renderer.models.get(model_index),
            }
            .map(|model| model.clone_untextured(device, queue));
            if let Some(mut selected_model) = selected_model {
                selected_model.get_transform_mut().scale.x *= 1.1;
                selected_model.get_transform_mut().scale.y *= 1.1;
                selected_model.get_transform_mut().scale.z *= 1.1;
                renderer.outline = Some(selected_model);
            }
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
