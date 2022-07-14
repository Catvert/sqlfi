use flume::{Receiver, Sender};

use eframe::{
    egui::{self, Frame, Layout, ScrollArea, Ui},
    emath::Align,
    epaint::Color32,
};
use egui::Window;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppData,
    db::{Message, MessageResponse},
    meta::{FetchResult, MetaColumn, MetaParam, MetaParamType, MetaParamValue, MetaQuery},
    ui::components::{self, icons, meta_table, sql_editor},
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
    execute_window: Option<ExecuteMetaQueryWindow>,

    #[serde(skip)]
    edit_window: Option<EditMetaQueryWindow>,

    #[serde(skip)]
    fetch_result: QueryState<FetchResult>,
}

impl Default for ViewData {
    fn default() -> Self {
        Self {
            show_left_panel: true,
            show_bottom_panel: true,
            bottom_tab: BottomTab::Query,
            query_history: vec![],
            query: String::new(),
            execute_window: None,
            edit_window: None,
            fetch_result: QueryState::Ready,
        }
    }
}

pub struct MetaQueriesView<'a> {
    pub rx: &'a Receiver<MessageResponse<MessageID>>,
    pub tx: &'a Sender<Message<MessageID>>,

    pub meta_queries: &'a mut IndexMap<String, MetaQuery>,

    pub schema: &'a str,

    pub data: &'a mut ViewData,
}

impl<'a> MetaQueriesView<'a> {
    pub fn from_app(app: &'a mut AppData, data: &'a mut ViewData) -> Self {
        MetaQueriesView {
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
                    ui.heading("Meta queries");
                });

                ui.separator();
                ScrollArea::both().show(ui, |ui| {
                    ui.vertical(|ui| {
                        for (query_id, query) in self.meta_queries.iter() {
                            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                                let btn = ui.button(&format!("{} {}", icons::ICON_RUN, query.name));

                                if btn.clicked() {
                                    self.data.execute_window =
                                        Some(ExecuteMetaQueryWindow::new(query.clone()));
                                }

                                if btn.clicked_by(egui::PointerButton::Secondary) {
                                    self.data.edit_window = Some(EditMetaQueryWindow::new(
                                        query_id.clone(),
                                        query.clone(),
                                    ));
                                }
                            });
                        }
                    });
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
                                        None,
                                    ),
                                );
                            }

                            if ui.button(icons::ICON_TRASH).clicked() {
                                self.data.fetch_result = QueryState::Ready;
                                self.data.query.clear();
                            }

                            ui.menu_button(icons::ICON_HISTORY, |ui| {});
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
            MessageResponse::FetchAllResult(_, res) => match res {
                Ok(res) => {
                    let results = res
                        .data
                        .into_iter()
                        .map(|(col, values)| {
                            let meta_col =
                                MetaColumn::default_sgdb_column(col.name(), col.r#type());

                            (meta_col, values)
                        })
                        .collect();

                    self.data.fetch_result = QueryState::Success(FetchResult {
                        num_rows: res.num_rows,
                        res: results,
                    });
                }
                Err(_) => todo!(),
            },
            MessageResponse::TablesResult(tables) => {}
        }
    }

    pub fn process_keybindings(&mut self, ui: &mut Ui) {
        let mut input = ui.input_mut();
        for (id, query) in self.meta_queries.iter() {
            if input.consume_key(query.hotkey.modifiers, query.hotkey.key) {
                self.data.execute_window = Some(ExecuteMetaQueryWindow::new(query.clone()));
            }
        }
    }
}

enum WindowAction {
    Continue,
    Close,
    Execute,
}

impl<'a> View for MetaQueriesView<'a> {
    fn init(&mut self) {}

    fn show(&mut self, ui: &mut Ui) {
        if let Ok(msg) = self.rx.try_recv() {
            self.process_db_response(msg);
        }

        self.process_keybindings(ui);

        if self.data.show_left_panel {
            self.show_left_panel(ui);
        }

        if self.data.show_bottom_panel {
            self.show_bottom_panel(ui);
        }

        self.show_central_panel(ui);

        let close = if let Some(window) = &mut self.data.execute_window {
            let mut open = true;
            if window.show(&mut open, ui) {
                let params = window
                    .params_values
                    .iter()
                    .map(|(key, (param, value))| match value {
                        MetaParamValue::Text(t) => t.clone(),
                        MetaParamValue::Boolean(_) => todo!(),
                        MetaParamValue::Number(_) => todo!(),
                        MetaParamValue::Decimal(_) => todo!(),
                    })
                    .collect();

                self.tx
                    .send(Message::FetchAll(
                        MessageID::FetchAllResult,
                        window.meta_query.query.clone(),
                        Some(params),
                    ))
                    .unwrap();
                open = false;
            }
            !open
        } else {
            false
        };

        if close {
            self.data.execute_window.take();
        }

        let action = if let Some(window) = &mut self.data.edit_window {
            let mut open = true;
            let mut action = WindowAction::Continue;
            if window.show(&mut open, ui) {
                action = WindowAction::Execute;
            }

            if !open {
                action = WindowAction::Close;
            }
            action
        } else {
            WindowAction::Continue
        };

        match action {
            WindowAction::Continue => {}
            WindowAction::Close => {
                self.data.edit_window.take();
            }
            WindowAction::Execute => {
                let EditMetaQueryWindow { key, meta_query } = self.data.edit_window.take().unwrap();

                *self.meta_queries.get_mut(&key).unwrap() = meta_query;
            }
        }
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

pub struct EditMetaQueryWindow {
    meta_query: MetaQuery,
    key: String,
}

impl EditMetaQueryWindow {
    fn new(key: String, meta_query: MetaQuery) -> Self {
        Self { key, meta_query }
    }
}

impl EditMetaQueryWindow {
    pub fn show(&mut self, open: &mut bool, ui: &mut Ui) -> bool {
        let submitted = Window::new(format!("Execute {}", self.meta_query.name))
            .open(open)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                let mut submitted = false;
                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(false)
                    .show(ui, |ui| {
                        for (id, param) in self.meta_query.params.iter_mut() {
                            ui.label(id);
                            match &mut param.default {
                                MetaParamValue::Text(text) => {
                                    ui.text_edit_singleline(text);
                                }
                                MetaParamValue::Boolean(_) => todo!(),
                                MetaParamValue::Number(_) => todo!(),
                                MetaParamValue::Decimal(_) => todo!(),
                            }
                        }
                    });

                ui.separator();

                components::sql_editor::code_view_ui(ui, &mut self.meta_query.query);

                ui.separator();
                ui.with_layout(Layout::top_down(eframe::emath::Align::Max), |ui| {
                    if ui.button("Add param").clicked() {
                        self.meta_query.params.insert(
                            "test".into(),
                            MetaParam {
                                id: "hello".into(),
                                r#type: MetaParamType::Text,
                                default: MetaParamValue::Text("hello".into()),
                            },
                        );
                    }
                    if ui.button("Save").clicked() {
                        submitted = true;
                    }
                });

                submitted
            });

        return if let Some(submitted) = submitted {
            Some(true) == submitted.inner
        } else {
            false
        };
    }
}

pub struct ExecuteMetaQueryWindow {
    meta_query: MetaQuery,
    params_values: IndexMap<String, (MetaParam, MetaParamValue)>,
}

impl ExecuteMetaQueryWindow {
    fn new(meta_query: MetaQuery) -> Self {
        let mut params_values = meta_query
            .params
            .iter()
            .map(|(id, param)| (id.clone(), (param.clone(), param.default.clone())))
            .collect();

        Self {
            meta_query,
            params_values,
        }
    }
}

impl ExecuteMetaQueryWindow {
    pub fn show(&mut self, open: &mut bool, ui: &mut Ui) -> bool {
        let submitted = Window::new(format!("Execute {}", self.meta_query.name))
            .open(open)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                let mut submitted = false;
                if self.params_values.is_empty() {
                    ui.label("No parameters");
                } else {
                    egui::Grid::new("my_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(false)
                        .show(ui, |ui| {
                            // ui.label("Name:").on_hover_text("The connection name.");
                            // ui.text_edit_singleline(&mut self.name);
                            // ui.end_row();

                            // ui.label("Connection string:").on_hover_text("The connection string to the database. E.g: 'mysql://{user}:{pwd}@localhost'");
                            // ui.text_edit_singleline(&mut self.uri);
                            // ui.end_row();

                            // ui.label("Database:").on_hover_text("The database to connect to.");
                            // ui.text_edit_singleline(&mut self.schema);
                            // ui.end_row();
                        });
                }

                ui.separator();

                components::sql_editor::code_view_ui_read_only(ui, &self.meta_query.query);

                ui.separator();
                ui.with_layout(Layout::top_down(eframe::emath::Align::Max), |ui| {
                    if ui.button("Execute").clicked() {
                        submitted = true;
                    }
                });

                submitted
            });

        return if let Some(submitted) = submitted {
            Some(true) == submitted.inner
        } else {
            false
        };
    }
}
