use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use eframe::{
    egui::{self, Button, Frame, Layout, RichText, ScrollArea, Ui},
    emath::{Align, Vec2},
    epaint::Color32,
};
use tokio::runtime::Builder;

use crate::{
    db::{
        sgdb::{connect, SGDBFetchResult, SGDBKind, SGDBTable},
        DBRelay, Message, MessageResponse,
    },
    meta::MetaView,
    ui::{
        components::{icons, sql_editor, top_menu},
        meta::MetaViewTable,
    },
};

enum QueryState<T> {
    Success(T),
    Waiting,
    Ready,
    Error(String),
}

impl<T> QueryState<T> {
    fn query<ID>(&mut self, tx: &Sender<Message<ID>>, msg: Message<ID>) {
        *self = QueryState::Waiting;
        tx.send(msg).unwrap();
    }
}

#[derive(Clone, Copy)]
pub enum MessageID {
    InsertRow,
    FetchAllResult,
}

#[derive(PartialEq, Eq)]
pub enum BottomTab {
    Query,
    Logs,
}

pub struct DBView {
    tx: Sender<Message<MessageID>>,
    handle_ui: Option<JoinHandle<()>>,
    handle_db: Option<JoinHandle<()>>,

    schema: String,

    fetch_result: Arc<Mutex<QueryState<(MetaView, SGDBFetchResult)>>>,

    query_history: Vec<String>,
    backtraces: Arc<Mutex<Vec<String>>>,
    query: String,

    tables: Arc<Mutex<QueryState<Vec<SGDBTable>>>>,

    bottom_tab: BottomTab,
}

impl DBView {
    pub fn spawn_view(uri: impl Into<String>, schema: impl Into<String>) -> Self {
        let (tx_ui, mut rx_db) = mpsc::channel();
        let (tx_db, mut rx_ui) = mpsc::channel();

        let uri = uri.into();
        let handle_db = thread::spawn(move || {
            let runtime = Builder::new_current_thread()
                .enable_all()
                .worker_threads(4)
                .build()
                .unwrap();

            runtime.block_on(async move {
                let sgdb = connect(SGDBKind::MySQL, &uri).await.unwrap();
                let db = DBRelay::new(sgdb, tx_db, rx_db).await;
                db.run().await;
            });
        });

        let mut backtraces = Arc::new(Mutex::new(vec![]));
        let backtraces_ui = backtraces.clone();

        let mut fetch_result = Arc::new(Mutex::new(QueryState::Ready));
        let fetch_ui = fetch_result.clone();

        let mut tables = Arc::new(Mutex::new(QueryState::Ready));
        let tables_ui = tables.clone();

        let handle_ui = thread::spawn(move || {
            while let Ok(msg) = rx_ui.recv() {
                match msg {
                    MessageResponse::FetchAllResult(id, res) => match id {
                        MessageID::InsertRow => {}
                        MessageID::FetchAllResult => {
                            match res {
                                Ok(res) => {
                                    let meta = MetaView::default_sgdb_result(&res);
                                    *fetch_ui.lock().unwrap() = QueryState::Success((meta, res));
                                }
                                Err(err) => {
                                    *fetch_ui.lock().unwrap() =
                                        QueryState::Error(format!("{}", err));
                                    backtraces_ui.lock().unwrap().push(format!("{}", err));
                                }
                            }
                            // *crows.lock().unwrap() = rows;
                        }
                    },
                    MessageResponse::Connected => todo!(),
                    MessageResponse::Closed => {
                        break;
                    }
                    MessageResponse::TablesResult(res) => match res {
                        Ok(res) => {
                            *tables_ui.lock().unwrap() = QueryState::Success(res);
                        }
                        Err(err) => {
                            *fetch_ui.lock().unwrap() = QueryState::Error(format!("{}", err));
                            backtraces_ui.lock().unwrap().push(format!("{}", err));
                        }
                    },
                }
            }
        });

        let schema = schema.into();

        tables.lock().unwrap().query(
            &tx_ui,
            Message::FetchTables {
                schema: schema.clone(),
            },
        );

        Self {
            handle_ui: Some(handle_ui),
            handle_db: Some(handle_db),
            tx: tx_ui,
            query: String::new(),
            backtraces,
            fetch_result,
            tables,
            schema,

            query_history: vec![],
            bottom_tab: BottomTab::Query,
        }
    }

    fn show_top_panel(&mut self, ui: &mut Ui) {
        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .default_height(100.)
            .frame(Frame::none())
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    top_menu(ui);
                    ui.menu_button("Actions", |ui| {
                        ui.button("E.g: Insert a new row");
                    });
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        ui.text_edit_singleline(&mut "Search..");
                        ui.separator();
                    });
                });
            });
    }

    fn show_left_panel(&mut self, ui: &mut Ui) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(300.)
            .max_width(400.)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Tables");
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui.button(icons::ICON_REFRESH).clicked() {
                            self.tables.lock().unwrap().query(
                                &self.tx,
                                Message::FetchTables {
                                    schema: self.schema.clone(),
                                },
                            );
                        }
                    });
                });

                ui.separator();
                ScrollArea::both().show(ui, |ui| match &*self.tables.lock().unwrap() {
                    QueryState::Success(res) => {
                        ui.vertical(|ui| {
                            for table in res.iter() {
                                let id = ui.make_persistent_id(&table.full_path);
                                ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                                    if ui
                                        .button(&format!(
                                            "{} {}",
                                            icons::ICON_TABLE,
                                            table.table_name
                                        ))
                                        .clicked()
                                    {
                                        self.query =
                                            format!("SELECT * FROM `{}`", table.table_name);

                                        self.fetch_result.lock().unwrap().query(
                                            &self.tx,
                                            Message::FetchAll(
                                                MessageID::FetchAllResult,
                                                self.query.clone(),
                                            ),
                                        );
                                    }
                                });
                            }
                        });
                    }
                    QueryState::Waiting => {
                        ui.label("Fetching tables..");
                    }
                    QueryState::Ready => {
                        ui.label("Ready to fetch tables");
                    }
                    QueryState::Error(_) => {
                        ui.colored_label(Color32::RED, "An error occurred while fetching tables");
                    }
                });
            });
    }

    pub fn show_bottom_panel(&mut self, ui: &mut Ui) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(100.)
            .height_range(80.0..=200.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.);
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.bottom_tab == BottomTab::Query, "Query")
                            .clicked()
                        {
                            self.bottom_tab = BottomTab::Query;
                        }
                        if ui
                            .selectable_label(self.bottom_tab == BottomTab::Logs, "Logs")
                            .clicked()
                        {
                            self.bottom_tab = BottomTab::Logs;
                        }

                        ui.separator();

                        ui.with_layout(Layout::right_to_left(), |ui| {
                            if ui.button(icons::ICON_RUN).clicked() {
                                self.fetch_result.lock().unwrap().query(
                                    &self.tx,
                                    Message::FetchAll(
                                        MessageID::FetchAllResult,
                                        self.query.clone(),
                                    ),
                                );
                            }

                            if ui.button(icons::ICON_TRASH).clicked() {
                                *self.fetch_result.lock().unwrap() = QueryState::Ready;
                                self.query.clear();
                            }

                            ui.menu_button(icons::ICON_HISTORY, |ui| {});
                        });
                    });

                    ui.separator();

                    match self.bottom_tab {
                        BottomTab::Query => {
                            ui.with_layout(
                                Layout::top_down(Align::Min).with_cross_justify(true),
                                |ui| {
                                    sql_editor::code_view_ui(ui, &mut self.query);
                                    ui.add_space(2.);
                                },
                            );
                        }
                        BottomTab::Logs => {
                            ScrollArea::both().show(ui, |ui| {
                                ui.with_layout(
                                    Layout::top_down(Align::Min).with_cross_justify(true),
                                    |ui| {
                                        for log in self.backtraces.lock().unwrap().iter() {
                                            ui.label(log);
                                        }
                                    },
                                );
                            });
                        }
                    }
                })
            });
    }

    pub fn show_central_panel(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default()
            .frame(Frame::group(ui.style()))
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    match &*self.fetch_result.lock().unwrap() {
                        QueryState::Success((meta, res)) => {
                            table_rows(ui, meta, res);
                        }
                        QueryState::Waiting => {
                            ui.colored_label(Color32::BLUE, "Loading..");
                        }
                        QueryState::Error(err) => {
                            ui.colored_label(
                                Color32::RED,
                                format!("An error has occurred: {}", err),
                            );
                        }
                        QueryState::Ready => {
                            ui.with_layout(
                                Layout::centered_and_justified(egui::Direction::TopDown),
                                |ui| {
                                    ui.heading("Waiting request..");
                                },
                            );
                        }
                    };
                });
            });
    }

    pub fn show(&mut self, ui: &mut Ui) {
        self.show_top_panel(ui);
        self.show_left_panel(ui);
        self.show_bottom_panel(ui);
        self.show_central_panel(ui);
    }
}

fn table_rows(ui: &mut egui::Ui, meta: &MetaView, res: &SGDBFetchResult) {
    use egui_extras::{Size, TableBuilder};

    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center))
        .column(Size::initial(20.))
        .columns(Size::remainder().at_least(100.), res.data.len())
        .resizable(true)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    let rich = RichText::new("Actions").underline();
                    ui.label(rich);
                });
            });
            for col in res.data.keys() {
                header.col(|ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        let rich = RichText::new(col.name()).underline();
                        ui.label(rich);
                    });
                });
            }
        })
        .body(|mut body| {
            body.rows(25.0, res.num_rows, |row_index, mut table_row| {
                table_row.col(|ui| {
                    ui.button(icons::ICON_EDIT);
                });
                for (col, values) in res.data.iter() {
                    let meta_column = meta.meta_column(col);
                    table_row.col(|ui| {
                        meta.table_cell(ui, meta_column, &values[row_index]);
                    });
                }
            });
        });
}

impl Drop for DBView {
    fn drop(&mut self) {
        self.tx.send(Message::Close).unwrap();

        if let Some(handle) = self.handle_db.take() {
            handle.join().unwrap();
        }

        if let Some(handle) = self.handle_ui.take() {
            handle.join().unwrap();
        }
    }
}
