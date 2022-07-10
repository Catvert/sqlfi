use eframe::egui::Ui;

pub mod icons;
pub mod sql_editor;

pub fn top_menu(ui: &mut Ui) {
    ui.menu_button("File", |ui| {
        if ui.button("Exit").clicked() {
            std::process::exit(0);
        }
    });
    ui.menu_button("Connections", |ui| {
        ui.radio(true, "Test");
        ui.radio(false, "Test2");

        ui.separator();
        ui.button("New connection..");
    });
    ui.menu_button("View", |ui| {
        ui.checkbox(&mut true, "Show left panel");
    });
    ui.separator();
}
