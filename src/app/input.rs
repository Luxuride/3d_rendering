use crate::app::Custom3d;
use crate::render::buffers::camera::CameraMovement;
use eframe::egui;
use eframe::egui::InputState;
use std::time::Duration;

impl Custom3d {
    pub fn handle_input(&mut self, input: &InputState, delta_time: &Duration) {
        if input.modifiers.shift {
            *self.camera.get_mov_speed_raw() = 10.0;
        }
        if input.key_down(egui::Key::W) {
            self.camera
                .process_keyboard_input(CameraMovement::Forward, delta_time);
        }
        if input.key_down(egui::Key::S) {
            self.camera
                .process_keyboard_input(CameraMovement::Backward, delta_time);
        }
        if input.key_down(egui::Key::A) {
            self.camera
                .process_keyboard_input(CameraMovement::Left, delta_time);
        }
        if input.key_down(egui::Key::D) {
            self.camera
                .process_keyboard_input(CameraMovement::Right, delta_time);
        }
        if input.key_down(egui::Key::Space) {
            // Move up
            self.camera
                .process_keyboard_input(CameraMovement::Up, delta_time);
        }
        if input.key_down(egui::Key::C) {
            // Move down
            self.camera
                .process_keyboard_input(CameraMovement::Down, delta_time);
        }
        if input.key_down(egui::Key::Q) {
            self.camera
                .process_keyboard_input(CameraMovement::FovUp, delta_time);
        }
        if input.key_down(egui::Key::E) {
            self.camera
                .process_keyboard_input(CameraMovement::FovDown, delta_time);
        }
        *self.camera.get_mov_speed_raw() = 1.0;
    }
}
