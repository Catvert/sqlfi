use flume::{Receiver, Sender};

use eframe::{
    egui::{self, Frame, Layout, ScrollArea, Ui},
    emath::Align,
    epaint::Color32,
};
use egui::{vec2, Align2, Window};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppData,
    config::{ConnectionConfig, SqlifeConfig},
    db::{Message, MessageResponse},
    meta::{FetchResult, MetaColumn, MetaParam, MetaParamType, MetaParamValue, MetaQuery},
    ui::components::{self, icons, meta_grid, meta_table, sql_editor},
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

    selected_index: usize,

    #[serde(skip)]
    right_panel: Option<RightPanel>,

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
            fetch_result: QueryState::Ready,
            right_panel: None,
            selected_index: 0,
        }
    }
}

pub enum RightPanel {
    EditMetaQuery(EditMetaQuery),
    ExecuteMetaQuery(ExecuteMetaQuery),
}

pub struct MetaQueriesView<'a> {
    pub rx: &'a Receiver<MessageResponse<MessageID>>,
    pub tx: &'a Sender<Message<MessageID>>,

    pub config: &'a mut SqlifeConfig,
    pub current_connection: Option<usize>,

    pub data: &'a mut ViewData,
}

impl<'a> MetaQueriesView<'a> {
    pub fn from_app(
        app: &'a mut AppData,
        data: &'a mut ViewData,
        config: &'a mut SqlifeConfig,
    ) -> Self {
        MetaQueriesView {
            data,

            config,

            rx: app.rx_sgdb.as_ref().unwrap(),
            tx: app.tx_sgdb.as_ref().unwrap(),
            current_connection: app.current_connection,
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
                        let con = &mut self.config.connections[self.current_connection.unwrap()];

                        for (query_id, query) in con.meta_queries.iter() {
                            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                                let btn = ui.button(&format!("{} {}", icons::ICON_RUN, query.name));

                                if btn.clicked() {
                                    self.data.right_panel = Some(RightPanel::ExecuteMetaQuery(
                                        ExecuteMetaQuery::new(query.clone()),
                                    ));
                                }

                                if btn.clicked_by(egui::PointerButton::Secondary) {
                                    self.data.right_panel = Some(RightPanel::EditMetaQuery(
                                        EditMetaQuery::new(query_id.clone(), query.clone()),
                                    ));
                                }
                            });
                        }
                    });
                });
            });
    }

    fn show_right_panel(&mut self, ui: &mut Ui) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(300.)
            .max_width(400.)
            .show_inside(ui, |ui| {
                let mut close = false;
                ui.horizontal(|ui| {
                    ui.heading("Execute {}");
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui.button(icons::ICON_CLOSE).clicked() {
                            close = true;
                        }
                    });
                });

                ui.separator();
                ScrollArea::both().show(ui, |ui| {
                    if let Some(right_panel) = &mut self.data.right_panel {
                        match right_panel {
                            RightPanel::EditMetaQuery(q) => {
                                let exec = q.show(ui);
                                if exec {
                                    let con = &mut self.config.connections
                                        [self.current_connection.unwrap()];
                                    close = true;
                                    *con.meta_queries.get_mut(&q.key).unwrap() = q.meta_query.clone();
                                }
                            }
                            RightPanel::ExecuteMetaQuery(q) => {
                                let exec = q.show(ui);

                                if exec {
                                    let params = q
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
                                            q.meta_query.query.clone(),
                                            Some(params),
                                        ))
                                        .unwrap();
                                    close = true;
                                }
                            }
                        }
                    }
                });

                if close {
                    self.data.right_panel.take();
                }
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
                            meta_grid::meta_grid(ui, meta, &mut self.data.selected_index);
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

        let con = &mut self.config.connections[self.current_connection.unwrap()];
        for (id, query) in con.meta_queries.iter() {
            if input.consume_key(query.hotkey.modifiers, query.hotkey.key) {
                self.data.right_panel = Some(RightPanel::ExecuteMetaQuery(ExecuteMetaQuery::new(
                    query.clone(),
                )));
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

        if self.data.right_panel.is_some() {
            self.show_right_panel(ui);
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

pub struct EditMetaQuery {
    meta_query: MetaQuery,
    key: String,
}

impl EditMetaQuery {
    fn new(key: String, meta_query: MetaQuery) -> Self {
        Self { key, meta_query }
    }
}

impl EditMetaQuery {
    pub fn show(&mut self, ui: &mut Ui) -> bool {
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
    }
}

pub struct ExecuteMetaQuery {
    request_focus: bool,
    meta_query: MetaQuery,
    params_values: IndexMap<String, (MetaParam, MetaParamValue)>,
}

impl ExecuteMetaQuery {
    fn new(meta_query: MetaQuery) -> Self {
        let mut params_values = meta_query
            .params
            .iter()
            .map(|(id, param)| (id.clone(), (param.clone(), param.default.clone())))
            .collect();

        Self {
            request_focus: true,
            meta_query,
            params_values,
        }
    }
}

impl ExecuteMetaQuery {
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        let mut submitted = false;
        if self.params_values.is_empty() {
            ui.label("No parameters");
        } else {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(false)
                .show(ui, |ui| {
                    for (index, (id, param)) in self.params_values.iter_mut().enumerate() {
                        ui.label(id);
                        let res = match &mut param.1 {
                            MetaParamValue::Text(text) => ui.text_edit_singleline(text),
                            MetaParamValue::Boolean(_) => todo!(),
                            MetaParamValue::Number(_) => todo!(),
                            MetaParamValue::Decimal(_) => todo!(),
                        };

                        if self.request_focus {
                            res.request_focus();
                            self.request_focus = false;
                        }
                        ui.end_row();
                    }
                });
        }

        ui.separator();

        components::sql_editor::code_view_ui_read_only(ui, &self.meta_query.query);

        ui.separator();
        ui.with_layout(Layout::top_down(eframe::emath::Align::Max), |ui| {
            let btn = ui.button("Execute");

            if self.request_focus {
                btn.request_focus();
                self.request_focus = false;
            }
            if btn.clicked() {
                submitted = true;
            }
        });

        submitted
    }
}
