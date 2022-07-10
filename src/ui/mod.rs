pub mod meta;
pub mod views;
pub mod components;

use eframe::egui::{self, ScrollArea, Style, Visuals};
use eframe::epaint::{Color32, Stroke};
use eframe::CreationContext;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::*;

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/VictorNerd.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

pub fn setup_style(cc: &CreationContext<'_>) {
    let mut visuals = Visuals::dark();
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(33, 37, 43);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 44, 52);
    visuals.widgets.active.bg_fill = Color32::from_rgb(97, 175, 239);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1., Color32::from_rgb(255, 255, 255));

    visuals.selection.bg_fill = Color32::from_rgb(40, 44, 52);
    visuals.selection.stroke = Stroke::new(1., Color32::from_rgb(97, 175, 239));

    visuals.faint_bg_color = Color32::from_rgb(36, 40, 47);
    visuals.extreme_bg_color = Color32::from_rgb(36, 40, 47);
    let mut style = Style {
        visuals,
        text_styles: [
            (Heading, FontId::new(25.0, Proportional)),
            (Name("Heading2".into()), FontId::new(23.0, Proportional)),
            (Name("Context".into()), FontId::new(20.0, Proportional)),
            (Body, FontId::new(18.0, Proportional)),
            (Monospace, FontId::new(16.0, Proportional)),
            (Button, FontId::new(16.0, Proportional)),
            (Small, FontId::new(14.0, Proportional)),
        ]
        .into(),
        ..Style::default()
    };

    setup_custom_fonts(&cc.egui_ctx);

    cc.egui_ctx.set_style(style);
}
