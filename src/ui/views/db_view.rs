use eframe::{
    egui::{self, Frame, Layout, ScrollArea, Ui},
    emath::Align,
    epaint::Color32,
};
use flume::{Sender, Receiver};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppData,
    meta::{MetaColumn, MetaQuery, FetchResult}, ui::components::{icons, sql_editor, meta_table},
};
use crate::db::{
        sgdb::SGDBTable,
        Message, MessageResponse,
    };

use super::{MessageID, QueryState, View};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
enum BottomTab {
    Query,
    Logs,
}


#[derive(Serialize, Deserialize)]
pub struct ViewData {
    show_left_panel: bool,
    show_bottom_panel: bool,
    bottom_tab: BottomTab,

    query_history: Vec<String>,
    query: String,

    #[serde(skip)]
    fetch_result: QueryState<FetchResult>,
    #[serde(skip)]
    tables: QueryState<Vec<SGDBTable>>,
}

impl Default for ViewData {
    fn default() -> Self {
        Self {
            show_left_panel: true,
            show_bottom_panel: true,
            bottom_tab: BottomTab::Query,
            query_history: vec![],
            query: String::new(),
            tables: QueryState::Ready,
            fetch_result: QueryState::Ready,
        }
    }
}

pub struct DBView<'a> {
    pub tx: &'a Sender<Message<MessageID>>,
    pub rx: &'a Receiver<MessageResponse<MessageID>>,

    pub meta_queries: &'a mut IndexMap<String, MetaQuery>,
    pub schema: &'a str,

    pub data: &'a mut ViewData,
}

impl<'a> DBView<'a> {
    pub fn from_app(app: &'a mut AppData, data: &'a mut ViewData) -> Self {
        DBView {
            meta_queries: &mut app.meta_queries,

            data,

            schema: &app.schema,
            rx: app.rx_sgdb.as_ref().unwrap(),
            tx: app.tx_sgdb.as_ref().unwrap(),
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
                            self.data.tables.query(
                                self.tx,
                                Message::FetchTables {
                                    schema: self.schema.to_string(),
                                },
                            );
                        }

                        // ui.menu_button(&self.schema, |ui| {
                        //     ui.radio(true, &self.schema);
                        // });
                    });
                });

                ui.separator();
                ScrollArea::both().show(ui, |ui| match &self.data.tables {
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
                                        self.data.query =
                                            format!("SELECT * FROM `{}`", table.table_name);

                                        self.data.fetch_result.query(
                                            self.tx,
                                            Message::FetchAll(
                                                MessageID::FetchAllResult,
                                                self.data.query.clone(),
                                                None
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

    fn show_bottom_panel(&mut self, ui: &mut Ui) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(100.)
            .height_range(80.0..=200.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.);
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.data.bottom_tab == BottomTab::Query, "Query")
                            .clicked()
                        {
                            self.data.bottom_tab = BottomTab::Query;
                        }
                        if ui
                            .selectable_label(self.data.bottom_tab == BottomTab::Logs, "Logs")
                            .clicked()
                        {
                            self.data.bottom_tab = BottomTab::Logs;
                        }

                        ui.separator();

                        ui.with_layout(Layout::right_to_left(), |ui| {
                            if ui.button(icons::ICON_RUN).clicked() {
                                self.data.fetch_result.query(
                                    self.tx,
                                    Message::FetchAll(
                                        MessageID::FetchAllResult,
                                        self.data.query.clone(),
                                        None
                                    ),
                                );
                            }

                            if ui.button(icons::ICON_TRASH).clicked() {
                                self.data.fetch_result = QueryState::Ready;
                                self.data.query.clear();
                            }

                            ui.menu_button(icons::ICON_HISTORY, |ui| {});

                            ui.separator();

                            if let QueryState::Success(res) = &self.data.fetch_result {
                                if ui.button(icons::ICON_ARROW_DOWN).clicked() {
                                    self.meta_queries.insert(
                                        "test".into(),
                                        MetaQuery::from_normal_query(
                                            "Test",
                                            self.data.query.clone(),
                                            &res,
                                        ),
                                    );
                                }
                            }
                        });
                    });

                    ui.separator();

                    match self.data.bottom_tab {
                        BottomTab::Query => {
                            ui.with_layout(
                                Layout::top_down(Align::Min).with_cross_justify(true),
                                |ui| {
                                    sql_editor::code_view_ui(ui, &mut self.data.query);
                                    ui.add_space(2.);
                                },
                            );
                        }
                        BottomTab::Logs => {
                            ScrollArea::both().show(ui, |ui| {
                                ui.with_layout(
                                    Layout::top_down(Align::Min).with_cross_justify(true),
                                    |ui| {
                                        // for log in self.backtraces.lock().iter() {
                                        //     ui.label(log);
                                        // }
                                    },
                                );
                            });
                        }
                    }
                })
            });
    }

    fn show_central_panel(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default()
            .frame(Frame::group(ui.style()))
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    match &self.data.fetch_result {
                        QueryState::Success(meta) => {
                            meta_table::meta_table(ui, meta);
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

    pub fn process_db_response(&mut self, message: MessageResponse<MessageID>) {
        match message {
            MessageResponse::FetchAllResult(_, res) => {
                match res {
                    Ok(res) => {
                        let results = res.data.into_iter()
                            .map(|(col, values)| {
                                let meta_col = MetaColumn::default_sgdb_column(col.name(), col.r#type());

                                (meta_col, values)
                            }).collect();

                        self.data.fetch_result = QueryState::Success(FetchResult { num_rows: res.num_rows, res: results });
                    },
                    Err(_) => todo!(),
                }
            },
            MessageResponse::TablesResult(tables) => {
                self.data.tables = match tables {
                    Ok(tables) => QueryState::Success(tables),
                    Err(err) => QueryState::Error(format!("{}", err))
                }
            },
        }
    }
}

impl<'a> View for DBView<'a> {
    fn init(&mut self) {
        self.data.tables.query(
            self.tx,
            Message::FetchTables {
                schema: self.schema.to_string(),
            },
        );
    }

    fn show(&mut self, ui: &mut Ui) {
        if let Ok(msg) = self.rx.try_recv() {
            self.process_db_response(msg);
        }

        if self.data.show_left_panel {
            self.show_left_panel(ui);
        }

        if self.data.show_bottom_panel {
            self.show_bottom_panel(ui);
        }

        self.show_central_panel(ui);
    }

    fn show_appbar(&mut self, ui: &mut Ui) {
        ui.menu_button("View", |ui| {
            ui.checkbox(&mut self.data.show_left_panel, "Show left panel");
            ui.checkbox(&mut self.data.show_bottom_panel, "Show bottom panel");
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
