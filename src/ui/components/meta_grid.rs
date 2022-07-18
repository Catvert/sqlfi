use egui::{Align, Color32, Frame, Layout, ScrollArea, Stroke};

use crate::{meta::FetchResult, ui::components::meta_table::MetaTableCell};

pub fn meta_grid(ui: &mut egui::Ui, res: &FetchResult, selected_index: &mut usize) {
    let size = ui.available_width();
    const cols: usize = 3;

    {
        if ui.input().key_pressed(egui::Key::J) {
            *selected_index += 1;
            if *selected_index >= res.num_rows {
                *selected_index = 0;
            }
        } else if ui.input().key_pressed(egui::Key::K) {
            if *selected_index == 0 {
                *selected_index = res.num_rows - 1;
            } else {
                *selected_index -= 1;
            }
        }

        let scroll = ui.input().scroll_delta.y;

        if scroll > 0. {
            if *selected_index == 0 {
                *selected_index = res.num_rows - 1;
            } else {
                *selected_index -= 1;
            }
        } else if scroll < 0. {
            *selected_index += 1;
            if *selected_index >= res.num_rows {
                *selected_index = 0;
            }
        }
    }

    ScrollArea::both().enable_scrolling(false).show(ui, |ui| {
        ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
            for row_index in 0..res.num_rows {
                Frame::group(ui.style())
                    .stroke(Stroke::new(
                        2.,
                        if *selected_index == row_index {
                            Color32::BLUE
                        } else {
                            Color32::BLACK
                        },
                    ))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (index, (col, values)) in res.res.iter().enumerate() {
                                ui.label(&col.name);

                                col.table_cell(ui, &values[row_index]);

                                if index + 1 < res.res.len() {
                                    ui.separator();
                                }
                            }

                            ui.with_layout(Layout::right_to_left(), |ui| {
                                // ui.label("yeah");
                            });
                        });
                    });

                if *selected_index == row_index {
                    ui.scroll_to_cursor(Some(Align::Center));
                }
            }
        });
    });
}
