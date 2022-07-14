pub mod db_view;
mod hello_view;
mod meta_queries_view;

use flume::Sender;

use eframe::{
    egui::Frame,
    egui::{Context, Layout, Ui, Window},
};

use crate::{
    app::AppData,
    config::ConnectionConfig,
    db::{sgdb::SGDBKind, Message},
    Sqlife,
};

pub trait QueryShareDB {
    fn query<ID>(&mut self, tx: Sender<Message<ID>>);
}

pub enum QueryState<T> {
    Success(T),
    Waiting,
    Ready,
    Error(String),
}

impl<T> Default for QueryState<T> {
    fn default() -> Self {
        Self::Ready
    }
}

impl<T> QueryState<T> {
    pub fn query<ID>(&mut self, tx: &Sender<Message<ID>>, msg: Message<ID>) {
        *self = QueryState::Waiting;
        tx.send(msg).unwrap();
    }
}

#[derive(Debug, Clone)]
pub enum MessageID {
    FetchAllResult,
    MetaQueryResult { meta_query_id: String },
}

pub enum CurrentView {
    HelloView,
    DBView(db_view::ViewData),
    MetaQueriesView(meta_queries_view::ViewData),
}

impl CurrentView {
    pub fn init(&mut self, app_data: &mut AppData) {
        match self {
            CurrentView::HelloView => hello_view::HelloView.init(),
            CurrentView::DBView(data) => db_view::DBView::from_app(app_data, data).init(),
            CurrentView::MetaQueriesView(data) => {
                meta_queries_view::MetaQueriesView::from_app(app_data, data).init()
            }
        };
    }

    fn show(&mut self, app_data: &mut AppData, ui: &mut Ui) {
        match self {
            CurrentView::HelloView => hello_view::HelloView.show(ui),
            CurrentView::DBView(data) => db_view::DBView::from_app(app_data, data).show(ui),
            CurrentView::MetaQueriesView(data) => {
                meta_queries_view::MetaQueriesView::from_app(app_data, data).show(ui)
            }
        };
    }

    fn show_appbar(&mut self, app_data: &mut AppData, ui: &mut Ui) {
        match self {
            CurrentView::HelloView => hello_view::HelloView.show_appbar(ui),
            CurrentView::DBView(view) => db_view::DBView::from_app(app_data, view).show_appbar(ui),
            CurrentView::MetaQueriesView(data) => {
                meta_queries_view::MetaQueriesView::from_app(app_data, data).show_appbar(ui)
            }
        }
    }
}

pub trait View {
    fn init(&mut self);
    fn show(&mut self, ui: &mut Ui);
    fn show_appbar(&mut self, ui: &mut Ui) {}
}

pub fn run(app: &mut Sqlife, ctx: &egui::Context) {
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
                        for (i, con) in app.data.connections.iter().enumerate() {
                            if ui
                                .radio(app.selected_connection() == Some(i), &con.name)
                                .clicked()
                            {
                                // app.switch_connection::<ConnectionSchema>(con.clone().into());
                                // app.switch_view(CurrentView::DBView(Default::default()));
                            }
                        }

                        ui.separator();
                        if ui.button("New connection..").clicked() {
                            app.data.new_connection_win.open = true;
                            ui.close_menu();
                        }
                    });
                    ui.separator();
                    if ui
                        .selectable_label(matches!(app.view, CurrentView::DBView(_)), "Tables")
                        .clicked()
                    {
                        app.switch_view(CurrentView::DBView(Default::default()));
                    }
                    if ui
                        .selectable_label(
                            matches!(app.view, CurrentView::MetaQueriesView(_)),
                            "Meta queries",
                        )
                        .clicked()
                    {
                        app.switch_view(CurrentView::MetaQueriesView(Default::default()));
                    }
                    ui.separator();

                    app.view.show_appbar(&mut app.data, ui);
                });
            });

        app.view.show(&mut app.data, ui);
    });

    app.data
        .new_connection_win
        .show(ctx, &mut app.data.connections);
}

#[derive(Default)]
pub struct NewConnectionWindow {
    pub open: bool,
    name: String,
    uri: String,
    kind: SGDBKind,
    schema: String,
}

impl NewConnectionWindow {
    pub fn show(&mut self, ctx: &Context, connections: &mut Vec<ConnectionConfig>) {
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
