use crate::app::Custom3d;
use eframe::egui;
use std::time::Duration;

impl Custom3d {
    pub fn top_panel(&mut self, delta_time: &Duration, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(format!("{:.2}ms", delta_time.as_secs_f32() * 1000.0));
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let last_ms = self.last_sync_ms.load(std::sync::atomic::Ordering::Relaxed);
                if last_ms > 0 {
                    ui.label(format!("last sync: {}ms", now_ms.saturating_sub(last_ms)));
                } else {
                    ui.label("last sync: -");
                }
                let peers = self.peers_count.load(std::sync::atomic::Ordering::Relaxed);
                ui.label(format!("peers: {}", peers));
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
