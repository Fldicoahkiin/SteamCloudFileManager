use crate::steam_api::CloudFile;
use egui;
use std::path::PathBuf;

#[derive(PartialEq, Clone, Copy)]
pub enum SortColumn {
    Name,
    Size,
    Time,
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum SortOrder {
    Ascending,
    Descending,
    #[default]
    None,
}

// 文件列表面板状态
#[derive(Default)]
pub struct FileListState {
    pub search_query: String,
    pub show_only_local: bool,
    pub show_only_cloud: bool,
    pub multi_select_mode: bool,
}

// 绘制文件表格项
pub fn draw_file_items_table(
    body: egui_extras::TableBody,
    files: &[CloudFile],
    selected_files: &mut Vec<usize>,
    state: &mut FileListState,
) {
    let row_height = 20.0;
    let filtered_files: Vec<(usize, &CloudFile)> = files
        .iter()
        .enumerate()
        .filter(|(_, file)| {
            if state.show_only_local && file.exists {
                return false;
            }
            if state.show_only_cloud && !file.exists {
                return false;
            }
            if !state.search_query.is_empty() {
                let query = state.search_query.to_lowercase();
                if !file.name.to_lowercase().contains(&query) {
                    return false;
                }
            }
            true
        })
        .collect();

    body.rows(row_height, filtered_files.len(), |mut row| {
        let row_index = row.index();
        if let Some((index, file)) = filtered_files.get(row_index) {
            let index = *index;
            let is_selected = selected_files.contains(&index);

            row.col(|ui| {
                let display_folder = if file.root_description.starts_with("CDP:") {
                    file.root_description
                        .split('|')
                        .nth(1)
                        .unwrap_or("CDP File")
                } else {
                    &file.root_description
                };
                ui.label(display_folder)
                    .on_hover_text(&file.root_description);
            });

            row.col(|ui| {
                #[allow(deprecated)]
                let response =
                    ui.add(egui::SelectableLabel::new(is_selected, &file.name).truncate());

                if response.clicked() {
                    let modifiers = ui.ctx().input(|i| i.modifiers);
                    let ctrl = modifiers.ctrl || modifiers.command;
                    let shift = modifiers.shift;

                    if state.multi_select_mode || ctrl {
                        if is_selected {
                            selected_files.retain(|&x| x != index);
                        } else {
                            selected_files.push(index);
                        }
                    } else if shift {
                        if let Some(&last) = selected_files.last() {
                            let (min, max) = if last < index {
                                (last, index)
                            } else {
                                (index, last)
                            };
                            for i in min..=max {
                                if !selected_files.contains(&i) {
                                    selected_files.push(i);
                                }
                            }
                        } else {
                            selected_files.push(index);
                        }
                    } else {
                        selected_files.clear();
                        selected_files.push(index);
                    }
                }
            });

            row.col(|ui| {
                ui.label(crate::utils::format_size(file.size));
            });

            row.col(|ui| {
                ui.label(file.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
            });

            row.col(|ui| {
                if file.exists {
                    ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "✓");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "✗");
                }
            });

            row.col(|ui| {
                if file.is_persisted {
                    ui.colored_label(egui::Color32::from_rgb(0, 150, 255), "✓");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "✗");
                }
            });
        }
    });
}

pub fn draw_file_drop_overlay(ui: &mut egui::Ui, ctx: &egui::Context) {
    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let painter = ui.painter();
        let rect = ui.available_rect_before_wrap();
        painter.rect_filled(
            rect,
            5.0,
            egui::Color32::from_rgba_premultiplied(0, 100, 200, 50),
        );
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "",
            egui::FontId::proportional(20.0),
            egui::Color32::WHITE,
        );
    }
}

// 文件列表排序动作
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortAction {
    None,
    SortByName,
    SortBySize,
    SortByTime,
}

// 文件列表面板状态
pub struct FileListPanelState<'a> {
    pub files: &'a [CloudFile],
    pub selected_files: &'a mut Vec<usize>,
    pub local_save_paths: &'a [(String, PathBuf)],
    pub search_query: &'a mut String,
    pub show_only_local: &'a mut bool,
    pub show_only_cloud: &'a mut bool,
    pub multi_select_mode: &'a mut bool,
    pub sort_column: Option<SortColumn>,
    pub sort_order: SortOrder,
    pub remote_ready: bool,
}

pub fn draw_complete_file_list_with_sort(
    ui: &mut egui::Ui,
    state: &mut FileListPanelState,
    on_open_folder: impl Fn(&PathBuf),
) -> SortAction {
    let mut sort_action = SortAction::None;

    if state.files.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("没有找到云文件");
        });
        return sort_action;
    }

    // 本地存档路径
    if !state.local_save_paths.is_empty() {
        ui.label("本地存档路径:");
        ui.horizontal_wrapped(|ui| {
            for (desc, path) in state.local_save_paths {
                let button_text = format!("📁 {}", desc);
                if ui
                    .button(button_text)
                    .on_hover_text(path.display().to_string())
                    .clicked()
                {
                    on_open_folder(path);
                }
            }
        });
        ui.separator();
    } else if state.remote_ready {
        ui.horizontal(|ui| {
            ui.label("本地存档路径:");
            ui.label("未找到（可能所有文件都仅在云端）");
        });
        ui.separator();
    }

    // 搜索和过滤
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(state.search_query)
                .desired_width(200.0)
                .hint_text("搜索文件..."),
        );

        if ui.button("清除搜索").clicked() {
            state.search_query.clear();
        }

        ui.separator();

        if ui
            .selectable_label(*state.show_only_local, "仅本地")
            .clicked()
        {
            *state.show_only_local = !*state.show_only_local;
            if *state.show_only_local {
                *state.show_only_cloud = false;
            }
        }

        if ui
            .selectable_label(*state.show_only_cloud, "仅云端")
            .clicked()
        {
            *state.show_only_cloud = !*state.show_only_cloud;
            if *state.show_only_cloud {
                *state.show_only_local = false;
            }
        }

        if ui
            .selectable_label(*state.multi_select_mode, "多选模式")
            .clicked()
        {
            *state.multi_select_mode = !*state.multi_select_mode;
        }
    });

    // 文件表格
    use egui_extras::{Column, TableBuilder};

    let available_height = ui.available_height();
    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(150.0))
        .column(Column::remainder().at_least(150.0))
        .column(Column::exact(80.0))
        .column(Column::exact(160.0))
        .column(Column::exact(40.0))
        .column(Column::exact(40.0))
        .max_scroll_height(available_height)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("文件夹");
            });
            header.col(|ui| {
                let name_btn = if state.sort_column == Some(SortColumn::Name) {
                    match state.sort_order {
                        SortOrder::Ascending => "文件名 ▲",
                        SortOrder::Descending => "文件名 ▼",
                        SortOrder::None => "文件名",
                    }
                } else {
                    "文件名"
                };
                if ui.button(name_btn).clicked() {
                    sort_action = SortAction::SortByName;
                }
            });
            header.col(|ui| {
                let size_btn = if state.sort_column == Some(SortColumn::Size) {
                    match state.sort_order {
                        SortOrder::Ascending => "文件大小 ▲",
                        SortOrder::Descending => "文件大小 ▼",
                        SortOrder::None => "文件大小",
                    }
                } else {
                    "文件大小"
                };
                if ui.button(size_btn).clicked() {
                    sort_action = SortAction::SortBySize;
                }
            });
            header.col(|ui| {
                let time_btn = if state.sort_column == Some(SortColumn::Time) {
                    match state.sort_order {
                        SortOrder::Ascending => "写入日期 ▲",
                        SortOrder::Descending => "写入日期 ▼",
                        SortOrder::None => "写入日期",
                    }
                } else {
                    "写入日期"
                };
                if ui.button(time_btn).clicked() {
                    sort_action = SortAction::SortByTime;
                }
            });
            header.col(|ui| {
                ui.label("本地");
            });
            header.col(|ui| {
                ui.label("云端");
            });
        })
        .body(|body| {
            let mut file_state = FileListState {
                search_query: state.search_query.clone(),
                show_only_local: *state.show_only_local,
                show_only_cloud: *state.show_only_cloud,
                multi_select_mode: *state.multi_select_mode,
            };
            draw_file_items_table(body, state.files, state.selected_files, &mut file_state);
        });

    sort_action
}
