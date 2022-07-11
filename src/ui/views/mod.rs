pub mod db_view;
mod hello_view;

use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex, MutexGuard,
    },
};

use eframe::{
    egui::Frame,
    egui::{Context, Layout, Ui, Window},
};

use crate::{
    app::AppData,
    config::ConnectionConfig,
    db::{
        sgdb::{SGDBFetchResult, SGDBKind, SGDBTable},
        Message, MessageResponse,
    },
    meta::MetaColumn,
    Sqlife,
};

use self::db_view::DBViewData;

pub struct ShareDB<T>(Arc<Mutex<T>>);

impl<T> ShareDB<T> {
    pub fn new(default: T) -> Self {
        Self(Arc::new(Mutex::new(default)))
    }

    pub fn duplicate(self) -> (Self, Self) {
        (self.share(), self)
    }

    pub fn lock(&self) -> MutexGuard<T> {
        self.0.lock().unwrap()
    }

    pub fn set(&self, v: T) {
        *self.0.lock().unwrap() = v;
    }

    pub fn share(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Clone for ShareDB<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Default> Default for ShareDB<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

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

#[derive(Debug, Clone, Copy)]
pub enum MessageID {
    FetchAllResult,
}

pub enum CurrentView {
    HelloView,
    DBView(db_view::DBViewData),
    MetaQueriesView,
}

impl CurrentView {
    pub fn init(&mut self, app_data: &mut AppData) {
        match self {
            CurrentView::HelloView => hello_view::DataView::from_app(app_data, &mut ()).init(),
            CurrentView::DBView(view) => db_view::DataView::from_app(app_data, view).init(),
            CurrentView::MetaQueriesView => todo!(),
        };
    }

    fn show(&mut self, app_data: &mut AppData, ui: &mut Ui) {
        match self {
            CurrentView::HelloView => hello_view::DataView::from_app(app_data, &mut ()).show(ui),
            CurrentView::DBView(view) => db_view::DataView::from_app(app_data, view).show(ui),
            CurrentView::MetaQueriesView => todo!(),
        };
    }

    fn show_appbar(&mut self, app_data: &mut AppData, ui: &mut Ui) {
        match self {
            CurrentView::HelloView => {
                hello_view::DataView::from_app(app_data, &mut ()).show_appbar(ui)
            }
            CurrentView::DBView(view) => {
                db_view::DataView::from_app(app_data, view).show_appbar(ui)
            }
            CurrentView::MetaQueriesView => todo!(),
        }
    }
}

pub trait View<'a, Data: Default> {
    fn from_app(app_data: &'a mut AppData, data: &'a mut Data) -> Self;

    fn init(&mut self);
    fn show(&mut self, ui: &mut Ui);
    fn show_appbar(&mut self, ui: &mut Ui);
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
                            {}
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
                        app.view = CurrentView::DBView(DBViewData::default());
                    }
                    if ui
                        .selectable_label(
                            matches!(app.view, CurrentView::MetaQueriesView),
                            "Meta queries",
                        )
                        .clicked()
                    {
                        app.view = CurrentView::MetaQueriesView;
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

#[derive(Clone, Default)]
pub struct DBData {
    tables: ShareDB<QueryState<Vec<SGDBTable>>>,
    fetch_result: ShareDB<QueryState<(HashMap<String, MetaColumn>, SGDBFetchResult)>>,
    backtraces: ShareDB<Vec<String>>,
}

pub fn process_db_response(rx_db: Receiver<MessageResponse<MessageID>>, ctx: DBData) {
    while let Ok(msg) = rx_db.recv() {
        match msg {
            MessageResponse::FetchAllResult(id, res) => match id {
                MessageID::FetchAllResult => match res {
                    Ok(res) => {
                        let meta_columns = res
                            .data
                            .keys()
                            .map(|col| {
                                (
                                    col.name().to_string(),
                                    MetaColumn::default_sgdb_column(col.r#type()),
                                )
                            })
                            .collect();
                        ctx.fetch_result
                            .set(QueryState::Success((meta_columns, res)));
                    }
                    Err(err) => {
                        ctx.fetch_result.set(QueryState::Error(format!("{}", err)));

                        ctx.backtraces.lock().push(format!("{}", err));
                    }
                },
            },
            MessageResponse::TablesResult(res) => {
                ctx.tables.set(
                    res.map(|res| QueryState::Success(res))
                        .unwrap_or_else(|err| QueryState::Error(format!("{}", err))),
                );
            }
            MessageResponse::Closed => {
                break;
            }
            _ => {}
        }
    }
}
