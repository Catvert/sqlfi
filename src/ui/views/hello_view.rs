use eframe::egui::{self, Layout};

use super::View;

pub struct DataView;

impl<'a> View<'a, ()> for DataView {
    fn init(&mut self) {}

    fn from_app(_: &'a mut crate::app::AppData, _: &'a mut ()) -> Self {
        Self
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(
        Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| {
            ui.heading("Welcome! To get started, open the 'Connections' menu and add a new connection.");
        },
    );
    }

    fn show_appbar(&mut self, ui: &mut egui::Ui) {}

}
