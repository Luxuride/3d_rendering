use std::time::Duration;
use eframe::egui;
use crate::app::Custom3d;

impl Custom3d {
    pub fn top_panel(&mut self, delta_time: &Duration, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(format!("{:.2}ms", delta_time.as_secs_f32() * 1000.0));
                ui.menu_button("About", |ui| {
                    let help_button = ui.button("Help");
                    if help_button.clicked() {
                        self.show_help = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }
}
