use eframe::egui::{self, Layout};

use super::View;

pub struct HelloView;

impl View for HelloView {
    fn init(&mut self) {}

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(
            Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.heading("Welcome! To get started, open the 'Connections' menu and add a new connection.");
            },
        );
    }
}
