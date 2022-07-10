use std::{
    future::Future,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle}, collections::HashMap,
};

use eframe::{
    egui::{self, Button, Frame, Layout, RichText, ScrollArea, Ui},
    emath::{Align, Vec2},
    epaint::Color32,
};
use log::info;

use crate::{
    db::{
        sgdb::{ConnectionSchema, SGDBFetchResult, SGDBTable},
        Message, MessageResponse, },
    meta::{MetaColumn, MetaQuery},
    ui::{
        components::{icons, sql_editor},
        meta::{MetaTableCell},
    },
};

use super::{QueryState, ShareDB, View};

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

    fetch_result: ShareDB<QueryState<(HashMap<String, MetaColumn>, SGDBFetchResult)>>,

    query_history: Vec<String>,
    backtraces: ShareDB<Vec<String>>,
    query: String,

    tables: ShareDB<QueryState<Vec<SGDBTable>>>,

    show_left_panel: bool,
    show_bottom_panel: bool,
    bottom_tab: BottomTab,

    schema: String,
}

impl DBView {
    pub fn spawn_view(con: ConnectionSchema) -> Self {
        let (tx_ui, rx_ui) = mpsc::channel();
        let (tx_db, rx_db) = mpsc::channel();

        let schema: String = con.schema().to_string();

        let handle_db = super::spawn_sgdb_relay(con, tx_db, rx_ui);

        let backtraces: (ShareDB<Vec<String>>, _) = ShareDB::default().duplicate();
        let fetch_result = ShareDB::default().duplicate();
        let tables = ShareDB::default().duplicate();

        let handle_ui = thread::spawn(move || {
            while let Ok(msg) = rx_db.recv() {
                match msg {
                    MessageResponse::FetchAllResult(id, res) => match id {
                        MessageID::InsertRow => {}
                        MessageID::FetchAllResult => {
                            match res {
                                Ok(res) => {
                                    let meta_columns = res.data.keys().map(|col| { (col.name().to_string(), MetaColumn::default_sgdb_column(col.r#type())) }).collect();
                                    fetch_result.1.set(QueryState::Success((meta_columns, res)));
                                }
                                Err(err) => {
                                    fetch_result.1.set(QueryState::Error(format!("{}", err)));

                                    backtraces.1.lock().push(format!("{}", err));
                                }
                            }
                            // *crows.lock().unwrap() = rows;
                        }
                    },
                    MessageResponse::Closed => {
                        break;
                    }
                    MessageResponse::TablesResult(res) => match res {
                        Ok(res) => {
                            tables.1.set(QueryState::Success(res));
                        }
                        Err(err) => {
                            fetch_result.1.set(QueryState::Error(format!("{}", err)));
                            backtraces.1.lock().push(format!("{}", err));
                        }
                    },
                }
            }
        });

        tables.0.lock().query(
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
            backtraces: backtraces.0,
            fetch_result: fetch_result.0,
            tables: tables.0,

            query_history: vec![],
            bottom_tab: BottomTab::Query,

            show_left_panel: true,
            show_bottom_panel: true,

            schema,
        }
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
                            self.tables.lock().query(
                                &self.tx,
                                Message::FetchTables {
                                    schema: self.schema.clone(),
                                },
                            );
                        }

                        // ui.menu_button(&self.schema, |ui| {
                        //     ui.radio(true, &self.schema);
                        // });
                    });
                });

                ui.separator();
                ScrollArea::both().show(ui, |ui| match &*self.tables.lock() {
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

                                        self.fetch_result.lock().query(
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
                                self.fetch_result.lock().query(
                                    &self.tx,
                                    Message::FetchAll(
                                        MessageID::FetchAllResult,
                                        self.query.clone(),
                                    ),
                                );
                            }

                            if ui.button(icons::ICON_TRASH).clicked() {
                                *self.fetch_result.lock() = QueryState::Ready;
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
                                        for log in self.backtraces.lock().iter() {
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
                    match &*self.fetch_result.lock() {
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
}

impl View for DBView {
    fn show(&mut self, ui: &mut Ui) {
        if self.show_left_panel {
            self.show_left_panel(ui);
        }

        if self.show_bottom_panel {
            self.show_bottom_panel(ui);
        }

        self.show_central_panel(ui);
    }

    fn show_appbar(&mut self, ui: &mut Ui) {
        ui.menu_button("View", |ui| {
            ui.checkbox(&mut self.show_left_panel, "Show left panel");
            ui.checkbox(&mut self.show_bottom_panel, "Show bottom panel");
        });
        ui.menu_button("Actions", |ui| {
            ui.button("E.g: Insert a new row");
        });
        ui.with_layout(Layout::right_to_left(), |ui| {
            ui.text_edit_singleline(&mut "Search..");
            ui.separator();
        });
    }
}

fn table_rows(ui: &mut egui::Ui, meta_columns: &HashMap<String, MetaColumn>, res: &SGDBFetchResult) {
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
                    let meta_column = meta_columns.get(col.name()).unwrap();
                    table_row.col(|ui| {
                        meta_column.table_cell(ui,  &values[row_index]);
                    });
                }
            });
        });
}

impl Drop for DBView {
    fn drop(&mut self) {
        self.tx.send(Message::Close).unwrap();

        info!("Dropping DB threads..");

        if let Some(handle) = self.handle_db.take() {
            handle.join().unwrap();
        }

        if let Some(handle) = self.handle_ui.take() {
            handle.join().unwrap();
        }

        info!("DB threads dropped");
    }
}
