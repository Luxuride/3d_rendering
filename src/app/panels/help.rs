use eframe::egui;
use crate::app::Custom3d;

impl Custom3d {
    pub fn help(&mut self, ctx: &egui::Context) {
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
}