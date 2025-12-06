use crate::file_tree::{FileTree, FileTreeNode};
use crate::steam_api::CloudFile;
use egui;
use egui_extras::{Column, TableBuilder};

const INDENT_WIDTH: f32 = 20.0; // 每层缩进宽度

// 格式化根文件夹显示（处理 CDP 格式）
fn format_root_description(root_description: &str) -> String {
    if root_description.starts_with("CDP:") {
        // CDP 格式：CDP:url|folder
        root_description
            .split('|')
            .nth(1)
            .unwrap_or("CDP File")
            .to_string()
    } else {
        root_description.to_string()
    }
}

// 收集节点下所有文件索引
fn collect_indices(node: &FileTreeNode, indices: &mut Vec<usize>) {
    match node {
        FileTreeNode::Folder { children, .. } => {
            for child in children {
                collect_indices(child, indices);
            }
        }
        FileTreeNode::File { index, .. } => {
            indices.push(*index);
        }
    }
}

// 渲染完整的文件树（使用表格布局）
pub fn render_file_tree(
    ui: &mut egui::Ui,
    tree: &mut FileTree,
    selected_files: &mut Vec<usize>,
    _files: &[CloudFile],
) {
    let available_height = ui.available_height();

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(150.0)) // 根文件夹
        .column(Column::remainder().at_least(200.0)) // 文件名（树状）
        .column(Column::exact(80.0)) // 文件大小
        .column(Column::exact(160.0)) // 写入日期
        .column(Column::exact(40.0)) // 本地
        .column(Column::exact(40.0)) // 云端
        .max_scroll_height(available_height)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("根文件夹");
            });
            header.col(|ui| {
                ui.label("文件名");
            });
            header.col(|ui| {
                ui.label("文件大小");
            });
            header.col(|ui| {
                ui.label("写入日期");
            });
            header.col(|ui| {
                ui.label("本地");
            });
            header.col(|ui| {
                ui.label("云端");
            });
        })
        .body(|mut body| {
            let root = tree.root_mut();
            if let Some(children) = root.children_mut() {
                render_tree_body(&mut body, children, selected_files, 0);
            }
        });
}

// 递归渲染树节点
fn render_tree_body(
    body: &mut egui_extras::TableBody,
    nodes: &mut [FileTreeNode],
    selected_files: &mut Vec<usize>,
    _indent_level: usize,
) {
    render_tree_body_recursive(body, nodes, selected_files, 0);
}

// 递归渲染树节点
fn render_tree_body_recursive(
    body: &mut egui_extras::TableBody,
    nodes: &mut [FileTreeNode],
    selected_files: &mut Vec<usize>,
    depth: usize,
) {
    let node_count = nodes.len();

    for (idx, node) in nodes.iter_mut().enumerate() {
        let _is_last_node = idx == node_count - 1;

        // 收集索引
        let indices_for_folder = if node.is_folder() {
            let mut indices = Vec::new();
            collect_indices(node, &mut indices);
            Some(indices)
        } else {
            None
        };

        match node {
            FileTreeNode::Folder {
                name,
                children,
                is_expanded,
                file_count,
                root_description,
                ..
            } => {
                let folder_name = name.clone();
                let count = *file_count;
                let expanded = *is_expanded;
                let root_desc = root_description.clone();

                // 渲染文件夹行
                body.row(18.0, |mut row| {
                    // 根文件夹列
                    row.col(|ui| {
                        let display_folder = format_root_description(&root_desc);
                        ui.label(display_folder).on_hover_text(&root_desc);
                    });

                    // 文件名列（带树状结构）
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // 简单缩进
                            ui.add_space(depth as f32 * INDENT_WIDTH);

                            // 展开/折叠箭头按钮
                            let arrow = if expanded { "▾" } else { "▸" };
                            if ui.small_button(arrow).clicked() {
                                *is_expanded = !*is_expanded;
                            }

                            // 文件夹图标和名称
                            let folder_icon = if expanded { "📂" } else { "📁" };
                            let folder_label =
                                format!("{} {} ({})", folder_icon, folder_name, count);
                            let response = ui.selectable_label(false, folder_label);

                            if response.clicked() {
                                // 选中文件夹下所有文件
                                if let Some(ref indices) = indices_for_folder {
                                    selected_files.clear();
                                    selected_files.extend(indices.clone());
                                }
                            }
                        });
                    });

                    // 文件大小列
                    row.col(|ui| {
                        ui.label("");
                    });

                    // 写入日期列
                    row.col(|ui| {
                        ui.label("");
                    });

                    // 本地列
                    row.col(|ui| {
                        ui.label("");
                    });

                    // 云端列
                    row.col(|ui| {
                        ui.label("");
                    });
                });

                // 如果展开，递归渲染子节点
                if *is_expanded && !children.is_empty() {
                    render_tree_body_recursive(body, children, selected_files, depth + 1);
                }
            }
            FileTreeNode::File {
                name, index, file, ..
            } => {
                let is_selected = selected_files.contains(index);
                let file_name = name.clone();
                let file_index = *index;

                // 渲染文件行
                body.row(18.0, |mut row| {
                    // 根文件夹列
                    row.col(|ui| {
                        let display_folder = format_root_description(&file.root_description);
                        ui.label(display_folder)
                            .on_hover_text(&file.root_description);
                    });

                    // 文件名列（带树状结构）
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // 缩进
                            ui.add_space(depth as f32 * INDENT_WIDTH);

                            // 箭头按钮
                            ui.add_space(
                                ui.spacing().button_padding.x * 2.0 + ui.spacing().icon_width,
                            );

                            // 文件名
                            let response = ui.selectable_label(is_selected, &file_name);

                            if response.clicked() {
                                let modifiers = ui.ctx().input(|i| i.modifiers);
                                let ctrl = modifiers.ctrl || modifiers.command;

                                if ctrl {
                                    // Ctrl 点击：切换选中状态
                                    if is_selected {
                                        selected_files.retain(|&x| x != file_index);
                                    } else {
                                        selected_files.push(file_index);
                                    }
                                } else {
                                    // 普通点击：单选
                                    selected_files.clear();
                                    selected_files.push(file_index);
                                }
                            }
                        });
                    });

                    // 文件大小列
                    row.col(|ui| {
                        ui.label(crate::utils::format_size(file.size));
                    });

                    // 写入日期列
                    row.col(|ui| {
                        ui.label(file.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
                    });

                    // 本地列
                    row.col(|ui| {
                        if file.exists {
                            ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "✓");
                        } else {
                            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "✗");
                        }
                    });

                    // 云端列
                    row.col(|ui| {
                        if file.is_persisted {
                            ui.colored_label(egui::Color32::from_rgb(0, 150, 255), "✓");
                        } else {
                            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "✗");
                        }
                    });
                });
            }
        }
    }
}
