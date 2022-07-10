#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod config;
mod db;
mod meta;
mod ui;

use config::ConnectionConfig;
use db::sgdb::{Connection, SGDBKind};
use eframe::egui::{self, Context, Window};
use eframe::egui::{Frame, Layout};
use eframe::CreationContext;
use ui::{
    setup_style,
    views::{DBView, HelloView},
};

use clap::{ArgEnum, Parser};
use ui::views::View;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
enum SGDBKindArgs {
    Mysql,
    Postgres,
    Sqlite,
}

impl Into<SGDBKind> for SGDBKindArgs {
    fn into(self) -> SGDBKind {
        match self {
            SGDBKindArgs::Mysql => SGDBKind::Mysql,
            SGDBKindArgs::Postgres => SGDBKind::Postgres,
            SGDBKindArgs::Sqlite => SGDBKind::Sqlite,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long, value_parser)]
    uri: Option<String>,
    #[clap(short, long, value_parser)]
    schema: Option<String>,
    #[clap(short, long, arg_enum, value_parser)]
    kind: Option<SGDBKindArgs>,
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

#[derive(Default)]
struct NewConnectionWindow {
    open: bool,
    name: String,
    uri: String,
    kind: SGDBKind,
    schema: String,
}

impl NewConnectionWindow {
    fn show(&mut self, ctx: &Context, connections: &mut Vec<ConnectionConfig>) {
        let close = Window::new("New connection")
            .open(&mut self.open)
            .resizable(false)
            .show(ctx, |ui| {
                let mut close = false;
                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(false)
                    .show(ui, |ui| {
                        ui.label("Name:").on_hover_text("The connection name.");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Connection string:").on_hover_text("The connection string to the database. E.g: 'mysql://{user}:{pwd}@localhost'");
                        ui.text_edit_singleline(&mut self.uri);
                        ui.end_row();

                        ui.label("Database:").on_hover_text("The database to connect to.");
                        ui.text_edit_singleline(&mut self.schema);
                        ui.end_row();
                    });
                ui.separator();
                ui.with_layout(Layout::top_down(eframe::emath::Align::Max), |ui| {
                    if ui.button("Add").clicked() {
                        connections.push(ConnectionConfig::new(&self.name, self.kind, &self.uri, &self.schema));
                        close = true;
                    }
                });

                close
            });

        if let Some(close) = close {
            if let Some(true) = close.inner {
                self.open = false;
            }
        }
    }
}

struct MyApp {
    selected_connection: Option<usize>,
    connections: Vec<ConnectionConfig>,
    view: Box<dyn View>,
    new_connection_win: NewConnectionWindow,
}

impl MyApp {
    pub fn new(cc: &CreationContext<'_>, args: Args) -> Self {
        setup_style(cc);

        let connection = ConnectionConfig::new(
            "default",
            args.kind.unwrap().into(),
            args.uri.unwrap(),
            args.schema.unwrap(),
        );

        let view = Box::new(DBView::spawn_view(connection.clone().into()));
        // } else {
        //     Box::new(HelloView::spawn_view())
        // };

        Self {
            view,
            connections: vec![connection],
            selected_connection: Some(0),
            new_connection_win: NewConnectionWindow::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::top("top_panel")
                .resizable(false)
                .default_height(100.)
                .frame(Frame::none())
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Exit").clicked() {
                                std::process::exit(0);
                            }
                        });
                        ui.menu_button("Connections", |ui| {
                            for (i, con) in self.connections.iter().enumerate() {
                                if ui
                                    .radio(self.selected_connection == Some(i), &con.name)
                                    .clicked()
                                {
                                    self.view = Box::new(DBView::spawn_view(con.clone().into()));

                                    self.selected_connection = Some(i);
                                }
                            }

                            ui.separator();
                            if ui.button("New connection..").clicked() {
                                self.new_connection_win.open = true;
                                ui.close_menu();
                            }
                        });
                        ui.separator();
                        self.view.show_appbar(ui);
                    });
                });

            self.view.show(ui);
        });

        self.new_connection_win.show(ctx, &mut self.connections);
    }
}
