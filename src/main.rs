#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod db;
mod ui;
mod meta;

use eframe::egui;
use eframe::CreationContext;
use ui::views::DBView;
use ui::setup_style;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long, value_parser)]
    connection: String,
    #[clap(short, long, value_parser)]
    schema: String
}

fn main() {
    pretty_env_logger::init();

    let args = Args::parse();

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "sqlfi",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc, args))),
    );
}

struct MyApp {
    db_view: DBView
}

impl MyApp {
    pub fn new(
        cc: &CreationContext<'_>,
        args: Args,
    ) -> Self {
        setup_style(cc);

        let db_view = DBView::spawn_view(format!("{}/{}", args.connection, args.schema), args.schema);

        Self {
            db_view
        }
    }
}

impl eframe::App for MyApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.db_view.show(ui);
        });
    }
}
