use egui::{RichText, Ui};

use crate::meta::{MetaColNumber, MetaColumnType, FetchResult};
use crate::ui::components::icons;
use crate::{db::sgdb::SGDBRowValue, meta::MetaColumn};

use eframe::{egui::Layout, emath::Align, epaint::Color32};

pub fn meta_table(ui: &mut egui::Ui, res: &FetchResult) {
    use egui_extras::{Size, TableBuilder};

    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center))
        .column(Size::initial(20.))
        .columns(Size::remainder().at_least(100.), res.res.len())
        .resizable(true)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    let rich = RichText::new("Actions").underline();
                    ui.label(rich);
                });
            });
            for col in res.res.keys() {
                header.col(|ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        let rich = RichText::new(&col.name).underline();
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

                for (col, values) in res.res.iter() {
                    table_row.col(|ui| {
                        col.table_cell(ui, &values[row_index]);
                    });
                }
            });
        });
}

pub trait MetaTableCell {
    fn table_cell(&self, ui: &mut Ui, field: &SGDBRowValue);
}

impl MetaTableCell for MetaColumn {
    #[inline]
    fn table_cell(&self, ui: &mut Ui, field: &SGDBRowValue) {
        let invalid_type = |ui: &mut Ui| {
            ui.label(format!(
                "Invalid meta column type ({:?}) for {:?}",
                self, field
            ))
        };

        if let SGDBRowValue::Null = field {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.colored_label(Color32::LIGHT_BLUE, "null");
            });
        } else {
            match &self.r#type {
                MetaColumnType::Text { color } => {
                    ui.with_layout(Layout::left_to_right(), |ui| {
                        if let SGDBRowValue::Text(text) = field {
                            ui.label(text);
                        } else {
                            invalid_type(ui);
                        }
                    });
                }
                MetaColumnType::CheckBox => {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        if let SGDBRowValue::Boolean(v) = field {
                            ui.checkbox(&mut v.clone(), "");
                        } else {
                            invalid_type(ui);
                        }
                    });
                }
                MetaColumnType::Number { variant } => {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        if let SGDBRowValue::Integer(v) = field {
                            match variant {
                                MetaColNumber::Simple => ui.label(v.to_string()),
                                MetaColNumber::Money => ui.label(format!("{:2}€", v.to_string())),
                            };
                        } else if let SGDBRowValue::UInteger(v) = field {
                            match variant {
                                MetaColNumber::Simple => ui.label(v.to_string()),
                                MetaColNumber::Money => ui.label(format!("{:2}€", v.to_string())),
                            };
                        } else if let SGDBRowValue::Decimal(v) = field {
                            match variant {
                                MetaColNumber::Simple => ui.label(v.to_string()),
                                MetaColNumber::Money => ui.label(format!("{:2}€", v.to_string())),
                            };
                        } else if let SGDBRowValue::Double(v) = field {
                            match variant {
                                MetaColNumber::Simple => ui.label(v.to_string()),
                                MetaColNumber::Money => ui.label(format!("{:2}€", v.to_string())),
                            };
                        } else {
                            invalid_type(ui);
                        }
                    });
                }
                MetaColumnType::DateTime { format } => {
                    if let SGDBRowValue::DateTime(v) = field {
                        ui.label(v.format(format).to_string());
                    } else {
                        invalid_type(ui);
                    }
                }
                MetaColumnType::Image(image_type) => todo!(),
                MetaColumnType::Binary => todo!(),
                MetaColumnType::Unknown => {
                    ui.colored_label(Color32::RED, "Unknown type");
                }
            }
        }
    }
}
