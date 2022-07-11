use std::{
    collections::HashMap,
    sync::mpsc::Sender,
};

use eframe::{
    egui::{self, Frame, Layout, RichText, ScrollArea, Ui},
    emath::Align,
    epaint::Color32,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppData,
    db::{
        sgdb::{SGDBFetchResult, SGDBTable},
        Message,
    },
    meta::MetaColumn,
    ui::{
        components::{icons, sql_editor},
        meta::MetaTableCell,
    },
};

use super::{MessageID, QueryState, ShareDB, View};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
enum BottomTab {
    Query,
    Logs,
}

#[derive(Serialize, Deserialize)]
pub struct DBViewData {
    show_left_panel: bool,
    show_bottom_panel: bool,
    bottom_tab: BottomTab,

    query_history: Vec<String>,
    query: String,
}

impl Default for DBViewData {
    fn default() -> Self {
        Self {
            show_left_panel: true,
            show_bottom_panel: true,
            bottom_tab: BottomTab::Query,
            query_history: vec![],
            query: String::new(),
        }
    }
}

pub struct DataView<'a> {
    pub tx: Sender<Message<MessageID>>,

    pub tables: &'a ShareDB<QueryState<Vec<SGDBTable>>>,
    pub fetch_result: &'a ShareDB<QueryState<(HashMap<String, MetaColumn>, SGDBFetchResult)>>,
    pub backtraces: &'a ShareDB<Vec<String>>,
    pub schema: &'a str,

    pub view: &'a mut DBViewData,
}

impl<'a> DataView<'a> {
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
                            // self.tables.lock().query(
                            //     &self.tx,
                            //     Message::FetchTables {
                            //         schema: self.schema.clone(),
                            //     },
                            // );
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
                                        self.view.query =
                                            format!("SELECT * FROM `{}`", table.table_name);

                                        self.fetch_result.lock().query(
                                            &self.tx,
                                            Message::FetchAll(
                                                MessageID::FetchAllResult,
                                                self.view.query.clone(),
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
                            .selectable_label(self.view.bottom_tab == BottomTab::Query, "Query")
                            .clicked()
                        {
                            self.view.bottom_tab = BottomTab::Query;
                        }
                        if ui
                            .selectable_label(self.view.bottom_tab == BottomTab::Logs, "Logs")
                            .clicked()
                        {
                            self.view.bottom_tab = BottomTab::Logs;
                        }

                        ui.separator();

                        ui.with_layout(Layout::right_to_left(), |ui| {
                            if ui.button(icons::ICON_RUN).clicked() {
                                self.fetch_result.lock().query(
                                    &self.tx,
                                    Message::FetchAll(
                                        MessageID::FetchAllResult,
                                        self.view.query.clone(),
                                    ),
                                );
                            }

                            if ui.button(icons::ICON_TRASH).clicked() {
                                *self.fetch_result.lock() = QueryState::Ready;
                                self.view.query.clear();
                            }

                            ui.menu_button(icons::ICON_HISTORY, |ui| {});
                        });
                    });

                    ui.separator();

                    match self.view.bottom_tab {
                        BottomTab::Query => {
                            ui.with_layout(
                                Layout::top_down(Align::Min).with_cross_justify(true),
                                |ui| {
                                    sql_editor::code_view_ui(ui, &mut self.view.query);
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

impl<'a> View<'a, DBViewData> for DataView<'a> {
    fn from_app(app: &'a mut AppData, data: &'a mut DBViewData) -> Self {
        DataView {
            tables: &app.db_data.tables,
            fetch_result: &app.db_data.fetch_result,
            backtraces: &app.db_data.backtraces,

            view: data,

            schema: &app.schema,
            tx: app.tx_sgdb.as_ref().unwrap().clone(),
        }
    }

    fn init(&mut self) {
        self.tables.lock().query(
            &self.tx,
            Message::FetchTables {
                schema: self.schema.to_string(),
            },
        );
    }

    fn show(&mut self, ui: &mut Ui) {
        if self.view.show_left_panel {
            self.show_left_panel(ui);
        }

        if self.view.show_bottom_panel {
            self.show_bottom_panel(ui);
        }

        self.show_central_panel(ui);
    }

    fn show_appbar(&mut self, ui: &mut Ui) {
        ui.menu_button("View", |ui| {
            ui.checkbox(&mut self.view.show_left_panel, "Show left panel");
            ui.checkbox(&mut self.view.show_bottom_panel, "Show bottom panel");
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

fn table_rows(
    ui: &mut egui::Ui,
    meta_columns: &HashMap<String, MetaColumn>,
    res: &SGDBFetchResult,
) {
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
                        meta_column.table_cell(ui, &values[row_index]);
                    });
                }
            });
        });
}
