use eframe::egui::{Layout, self};

use super::View;

pub struct HelloView {}

impl HelloView {
    pub fn spawn_view() -> Self {
        HelloView {}
    }
}

impl View for HelloView {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        ui.with_layout(
            Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.heading("Welcome! To get started, open the 'Connections' menu and add a new connection.");
            },
        );
    }

    fn show_appbar(&mut self, ui: &mut eframe::egui::Ui) {}
}
