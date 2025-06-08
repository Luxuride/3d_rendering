use crate::render::buffers::camera::{Camera, CameraBuilder, CameraMovement};
use crate::render::buffers::transform::Transform;
use crate::render::model::Model;
use crate::render::renderer::{RendererCallback, RendererRenderResources};
use cgmath::Point3;
use eframe::egui::Label;
use eframe::{egui, egui_wgpu};
use std::sync::atomic::AtomicU8;
use std::sync::{Arc, RwLock};

pub struct Custom3d {
    camera: Camera,
    renderer: Arc<RwLock<RendererRenderResources>>,
    selected_model: SelectedModel,
    loading: Arc<AtomicU8>,
    show_help: bool,
}

#[derive(PartialEq, Clone, Copy)]
enum SelectedModel {
    Wireframe(usize),
    Model(usize),
    None,
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
            selected_model: SelectedModel::None,
            loading: Arc::new(AtomicU8::new(0)),
            show_help: false,
        })
    }
}

impl eframe::App for Custom3d {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if i.modifiers.shift {
                *self.camera.get_mov_speed_raw() = 1.0;
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
            *self.camera.get_mov_speed_raw() = 0.1;
        });
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("About", |ui| {
                    let help_button = ui.button("Help");
                    if help_button.clicked() {
                        self.show_help = true;
                        ui.close_menu();
                    }
                });
            });
        });
        if self.show_help {
            egui::Window::new("Help")
                .resizable(false)
                .collapsible(false)
                .open(&mut self.show_help)
                .show(ctx, |ui| {
                    ui.label("LMB down: Look around");
                    ui.label("W: Forward");
                    ui.label("A: Left");
                    ui.label("S: Back");
                    ui.label("D: Right");
                    ui.label("Space: Up");
                    ui.label("C: Down");
                    ui.label("Shift: Go fast");
                    ui.label("Q: FOV Up");
                    ui.label("E: FOV Down");
                });
        }
        egui::SidePanel::right("right_panel")
            .resizable(false)
            .show(ctx, |ui| {
                self.handle_right_panel(ui);
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

    pub fn handle_right_panel(&mut self, ui: &mut egui::Ui) {
        let renderer = &mut self.renderer.write().unwrap();
        ui.radio_value(
            &mut self.selected_model,
            SelectedModel::None,
            "None".to_string(),
        );
        for (model_index, _) in renderer.wireframe_models.iter().enumerate() {
            ui.radio_value(
                &mut self.selected_model,
                SelectedModel::Wireframe(model_index),
                format!("Wireframe {}", model_index),
            );
        }
        for (model_index, _) in renderer.models.iter().enumerate() {
            ui.radio_value(
                &mut self.selected_model,
                SelectedModel::Model(model_index),
                format!("Model {}", model_index),
            );
        }
        let selected_model = match self.selected_model {
            SelectedModel::Wireframe(model_index) => renderer.wireframe_models.get_mut(model_index),
            SelectedModel::Model(model_index) => renderer.models.get_mut(model_index),
            SelectedModel::None => None,
        };
        if let Some(selected_model) = selected_model {
            ui.add(egui::DragValue::new(
                &mut selected_model.get_transform_mut().position.x,
            ).speed(0.1));
            ui.add(egui::DragValue::new(
                &mut selected_model.get_transform_mut().position.y,
            ).speed(0.1));
            ui.add(egui::DragValue::new(
                &mut selected_model.get_transform_mut().position.z,
            ).speed(0.1));
            ui.add(egui::DragValue::new(
                &mut selected_model.get_transform_mut().scale.x,
            ).speed(0.1));
            ui.add(egui::DragValue::new(
                &mut selected_model.get_transform_mut().scale.y,
            ).speed(0.1));
            ui.add(egui::DragValue::new(
                &mut selected_model.get_transform_mut().scale.z,
            ).speed(0.1));
        }
        let device = &renderer.wgpu_render_state.device.clone();
        let queue = &renderer.wgpu_render_state.queue.clone();
        let selected_model = match self.selected_model {
            SelectedModel::Wireframe(model_index) => renderer.wireframe_models.get(model_index),
            SelectedModel::Model(model_index) => renderer.models.get(model_index),
            SelectedModel::None => None,
        }
        .map(|model| model.clone_untextured(device, queue));
        if let Some(selected_model) = selected_model {
            renderer.outline = Some(selected_model);
        } else {
            renderer.outline = None;
        }
    }
}
