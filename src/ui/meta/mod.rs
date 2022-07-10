use eframe::{
    egui::{Layout, Ui},
    emath::Align,
    epaint::Color32,
};

use crate::{
    db::sgdb::SGDBRowValue,
    meta::{MetaColNumber, MetaColumn},
};

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
            match self {
                MetaColumn::Text { color } => {
                    ui.with_layout(Layout::left_to_right(), |ui| {
                        if let SGDBRowValue::Text(text) = field {
                            ui.label(text);
                        } else {
                            invalid_type(ui);
                        }
                    });
                }
                MetaColumn::CheckBox => {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        if let SGDBRowValue::Boolean(v) = field {
                            ui.checkbox(&mut v.clone(), "");
                        } else {
                            invalid_type(ui);
                        }
                    });
                }
                MetaColumn::Number { variant } => {
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
                MetaColumn::DateTime { format } => {
                    if let SGDBRowValue::DateTime(v) = field {
                        ui.label(v.format(format).to_string());
                    } else {
                        invalid_type(ui);
                    }
                }
                MetaColumn::Binary => todo!(),
                MetaColumn::Unknown => {
                    ui.colored_label(Color32::RED, "Unknown type");
                }
            }
        }
    }
}
