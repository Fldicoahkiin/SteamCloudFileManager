use crate::conflict::SyncStatus;
use crate::file_tree::{FileTree, FileTreeNode};
use crate::i18n::I18n;
use crate::icons;
use crate::steam_api::CloudFile;
use egui;
use egui_extras::{Column, TableBuilder};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

const INDENT_WIDTH: f32 = 20.0; // 每层缩进宽度

// 树状视图状态
pub struct TreeViewState<'a> {
    pub search_query: &'a mut String,
    pub show_only_local: &'a mut bool,
    pub show_only_cloud: &'a mut bool,
    pub last_selected_index: &'a mut Option<usize>,
}

// 递归渲染时的只读上下文
struct TreeRenderContext<'a> {
    search_query: &'a str,
    show_only_local: bool,
    show_only_cloud: bool,
    sync_status_map: &'a HashMap<String, SyncStatus>,
}

// 树节点渲染的可变上下文
struct TreeBodyContext<'a> {
    search_query: &'a str,
    show_only_local: bool,
    show_only_cloud: bool,
    last_selected_index: &'a mut Option<usize>,
    sync_status_map: &'a HashMap<String, SyncStatus>,
}

// 绘制树状线条
fn draw_tree_lines(ui: &mut egui::Ui, depth: usize, is_last: bool, parent_is_last: &[bool]) -> f32 {
    if depth == 0 {
        return 0.0;
    }

    let line_color = crate::ui::theme::muted_color(ui.ctx());
    let painter = ui.painter();
    let rect = ui.available_rect_before_wrap();
    let y_mid = rect.center().y; // 行的中心点
    let base_x = rect.min.x;

    // 绘制父级的垂直线
    for (level, &parent_last) in parent_is_last.iter().enumerate() {
        if !parent_last {
            let x = base_x + (level as f32 + 0.5) * INDENT_WIDTH;
            painter.line_segment(
                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                egui::Stroke::new(1.0, line_color),
            );
        }
    }

    // 绘制当前节点的连接线
    let current_level = depth - 1;
    let x = base_x + (current_level as f32 + 0.5) * INDENT_WIDTH;

    // 垂直线
    if is_last {
        // 最后一个节点
        painter.line_segment(
            [egui::pos2(x, rect.min.y), egui::pos2(x, y_mid)],
            egui::Stroke::new(1.0, line_color),
        );
    } else {
        // 非最后节点
        painter.line_segment(
            [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
            egui::Stroke::new(1.0, line_color),
        );
    }

    // 水平线
    let h_end = base_x + depth as f32 * INDENT_WIDTH;
    painter.line_segment(
        [egui::pos2(x, y_mid), egui::pos2(h_end, y_mid)],
        egui::Stroke::new(1.0, line_color),
    );

    depth as f32 * INDENT_WIDTH
}

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

// 收集当前可见的所有文件索引（用于 Shift 选择）
fn collect_visible_file_indices(
    nodes: &[FileTreeNode],
    show_only_local: bool,
    show_only_cloud: bool,
) -> Vec<usize> {
    let mut indices = Vec::new();
    collect_visible_indices_recursive(nodes, show_only_local, show_only_cloud, &mut indices);
    indices
}

fn collect_visible_indices_recursive(
    nodes: &[FileTreeNode],
    show_only_local: bool,
    show_only_cloud: bool,
    indices: &mut Vec<usize>,
) {
    for node in nodes {
        match node {
            FileTreeNode::Folder {
                children,
                is_expanded,
                ..
            } => {
                // 只有展开的文件夹才递归处理
                if *is_expanded {
                    collect_visible_indices_recursive(
                        children,
                        show_only_local,
                        show_only_cloud,
                        indices,
                    );
                }
            }
            FileTreeNode::File { index, file, .. } => {
                // 检查文件是否匹配筛选条件
                if matches_filter(file, show_only_local, show_only_cloud) {
                    indices.push(*index);
                }
            }
        }
    }
}

// 检查节点是否匹配搜索条件
fn matches_search(node: &FileTreeNode, search_query: &str) -> bool {
    if search_query.is_empty() {
        return true;
    }

    let name = node.name();

    // 尝试作为正则表达式匹配
    if let Ok(regex) = Regex::new(search_query) {
        regex.is_match(name)
    } else {
        // 如果不是有效的正则表达式，使用普通字符串匹配（不区分大小写）
        name.to_lowercase().contains(&search_query.to_lowercase())
    }
}

// 检查节点或其子节点是否匹配搜索条件
fn node_or_children_match(node: &FileTreeNode, search_query: &str) -> bool {
    if search_query.is_empty() {
        return true;
    }

    // 检查当前节点
    if matches_search(node, search_query) {
        return true;
    }

    // 检查子节点
    if let FileTreeNode::Folder { children, .. } = node {
        for child in children {
            if node_or_children_match(child, search_query) {
                return true;
            }
        }
    }

    false
}

// 检查文件是否匹配筛选条件
fn matches_filter(file: &CloudFile, show_only_local: bool, show_only_cloud: bool) -> bool {
    if show_only_local {
        // 仅本地：显示本地存在但云端不存在的文件
        return file.exists && !file.is_persisted;
    }
    if show_only_cloud {
        // 仅云端：显示云端存在但本地不存在的文件
        return file.is_persisted && !file.exists;
    }
    true
}

// 检查节点或其子节点是否匹配筛选条件（用于过滤空文件夹）
fn node_or_children_match_filter(
    node: &FileTreeNode,
    show_only_local: bool,
    show_only_cloud: bool,
) -> bool {
    // 如果没有筛选条件，所有节点都匹配
    if !show_only_local && !show_only_cloud {
        return true;
    }

    match node {
        FileTreeNode::Folder { children, .. } => {
            // 文件夹：递归检查是否有任何子节点匹配
            for child in children {
                if node_or_children_match_filter(child, show_only_local, show_only_cloud) {
                    return true;
                }
            }
            false
        }
        FileTreeNode::File { file, .. } => {
            // 文件：直接检查是否匹配筛选条件
            matches_filter(file, show_only_local, show_only_cloud)
        }
    }
}

// 文件树渲染参数
pub struct FileTreeRenderParams<'a> {
    pub tree: &'a mut FileTree,
    pub selected_files: &'a mut Vec<usize>,
    pub local_save_paths: &'a [(String, PathBuf)],
    pub remote_ready: bool,
    pub state: &'a mut TreeViewState<'a>,
    pub i18n: &'a I18n,
    pub sync_status_map: &'a HashMap<String, SyncStatus>, // 文件名 -> 同步状态
}

// 渲染完整的文件树
pub fn render_file_tree(ui: &mut egui::Ui, params: FileTreeRenderParams) {
    let FileTreeRenderParams {
        tree,
        selected_files,
        local_save_paths,
        remote_ready,
        state,
        i18n,
        sync_status_map,
    } = params;
    // 本地存档路径
    if !local_save_paths.is_empty() {
        ui.label(i18n.local_save_path());
        ui.horizontal_wrapped(|ui| {
            for (desc, path) in local_save_paths {
                let button_text = format!("{} {}", icons::FOLDER, desc);
                if ui
                    .button(button_text)
                    .on_hover_text(path.display().to_string())
                    .clicked()
                {
                    crate::file_manager::open_folder(path);
                }
            }
        });
        ui.separator();
    } else if remote_ready {
        ui.horizontal(|ui| {
            ui.label(i18n.local_save_path());
            ui.label(i18n.local_save_path_not_found());
        });
        ui.separator();
    }

    // 搜索和筛选
    ui.horizontal(|ui| {
        ui.label(icons::MAGNIFYING_GLASS);
        ui.add(
            egui::TextEdit::singleline(state.search_query)
                .desired_width(200.0)
                .hint_text(i18n.search_files_placeholder()),
        );

        if ui.button(i18n.clear()).clicked() {
            state.search_query.clear();
        }

        ui.separator();

        if ui
            .selectable_label(*state.show_only_local, i18n.only_local())
            .on_hover_text(i18n.only_local_tooltip())
            .clicked()
        {
            *state.show_only_local = !*state.show_only_local;
            if *state.show_only_local {
                *state.show_only_cloud = false;
            }
        }

        if ui
            .selectable_label(*state.show_only_cloud, i18n.only_cloud())
            .on_hover_text(i18n.only_cloud_tooltip())
            .clicked()
        {
            *state.show_only_cloud = !*state.show_only_cloud;
            if *state.show_only_cloud {
                *state.show_only_local = false;
            }
        }
    });

    ui.separator();

    let available_height = ui.available_height();

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(130.0)) // 根文件夹
        .column(Column::remainder().at_least(200.0)) // 文件名（树状）
        .column(Column::exact(80.0)) // 文件大小
        .column(Column::exact(140.0)) // 写入日期
        .column(Column::exact(40.0)) // 本地
        .column(Column::exact(40.0)) // 云端
        .column(Column::exact(70.0)) // 状态
        .max_scroll_height(available_height)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label(i18n.root_folder());
            });
            header.col(|ui| {
                ui.label(i18n.file_name());
            });
            header.col(|ui| {
                ui.label(i18n.file_size());
            });
            header.col(|ui| {
                ui.label(i18n.write_date());
            });
            header.col(|ui| {
                ui.label(i18n.local());
            });
            header.col(|ui| {
                ui.label(i18n.cloud());
            });
            header.col(|ui| {
                ui.label(i18n.column_status());
            });
        })
        .body(|mut body| {
            let root = tree.root_mut();
            if let Some(children) = root.children_mut() {
                let mut ctx = TreeBodyContext {
                    search_query: state.search_query,
                    show_only_local: *state.show_only_local,
                    show_only_cloud: *state.show_only_cloud,
                    last_selected_index: state.last_selected_index,
                    sync_status_map,
                };
                render_tree_body(&mut body, children, selected_files, &mut ctx);
            }
        });
}

// 递归渲染树节点
fn render_tree_body(
    body: &mut egui_extras::TableBody,
    nodes: &mut [FileTreeNode],
    selected_files: &mut Vec<usize>,
    ctx: &mut TreeBodyContext,
) {
    // 先收集当前可见的所有文件索引
    let visible_indices =
        collect_visible_file_indices(nodes, ctx.show_only_local, ctx.show_only_cloud);

    let render_ctx = TreeRenderContext {
        search_query: ctx.search_query,
        show_only_local: ctx.show_only_local,
        show_only_cloud: ctx.show_only_cloud,
        sync_status_map: ctx.sync_status_map,
    };
    render_tree_body_recursive(
        body,
        nodes,
        selected_files,
        1, // 从 depth=1 开始，让根级节点也显示树状线
        &[],
        &render_ctx,
        ctx.last_selected_index,
        &visible_indices,
    );
}

// 递归渲染树节点
#[allow(clippy::too_many_arguments)]
fn render_tree_body_recursive(
    body: &mut egui_extras::TableBody,
    nodes: &mut [FileTreeNode],
    selected_files: &mut Vec<usize>,
    depth: usize,
    parent_is_last: &[bool],
    ctx: &TreeRenderContext,
    last_selected_index: &mut Option<usize>,
    visible_indices: &[usize],
) {
    // 先计算过滤后最后一个可见节点的索引
    let last_visible_idx = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            node_or_children_match(node, ctx.search_query)
                && node_or_children_match_filter(node, ctx.show_only_local, ctx.show_only_cloud)
        })
        .map(|(idx, _)| idx)
        .next_back();

    for (idx, node) in nodes.iter_mut().enumerate() {
        // 检查节点是否匹配搜索条件
        if !node_or_children_match(node, ctx.search_query) {
            continue;
        }

        // 检查节点是否匹配筛选条件（仅本地/仅云端）
        if !node_or_children_match_filter(node, ctx.show_only_local, ctx.show_only_cloud) {
            continue;
        }

        // 判断是否是过滤后的最后一个可见节点
        let is_last_node = last_visible_idx == Some(idx);

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

                    // 文件名列
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // 绘制树状线条
                            let indent = draw_tree_lines(ui, depth, is_last_node, parent_is_last);
                            ui.add_space(indent);

                            // 展开/折叠箭头按钮
                            let arrow = if expanded {
                                icons::ARROW_DOWN
                            } else {
                                icons::ARROW_RIGHT
                            };
                            if ui.small_button(arrow).clicked() {
                                *is_expanded = !*is_expanded;
                            }

                            // 文件夹图标和名称
                            let folder_icon = if expanded {
                                icons::FOLDER_OPEN
                            } else {
                                icons::FOLDER
                            };
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
                    row.col(|_ui| {});

                    // 写入日期列
                    row.col(|_ui| {});

                    // 本地列
                    row.col(|_ui| {});

                    // 云端列
                    row.col(|_ui| {});

                    // 状态列（文件夹不显示状态）
                    row.col(|_ui| {});
                });

                if *is_expanded && !children.is_empty() {
                    let mut new_parent_is_last = parent_is_last.to_vec();
                    new_parent_is_last.push(is_last_node);
                    render_tree_body_recursive(
                        body,
                        children,
                        selected_files,
                        depth + 1,
                        &new_parent_is_last,
                        ctx,
                        last_selected_index,
                        visible_indices,
                    );
                }
            }
            FileTreeNode::File {
                name, index, file, ..
            } => {
                // 检查文件是否匙配筛选条件
                if !matches_filter(file, ctx.show_only_local, ctx.show_only_cloud) {
                    continue;
                }

                let is_selected = selected_files.contains(index);
                let file_name = name.clone();
                let file_index = *index;

                // 获取同步状态
                let sync_status = ctx.sync_status_map.get(&file.name).copied();

                // 渲染文件行
                body.row(18.0, |mut row| {
                    // 根文件夹列
                    row.col(|ui| {
                        let display_folder = format_root_description(&file.root_description);
                        ui.label(display_folder)
                            .on_hover_text(&file.root_description);
                    });

                    // 文件名列
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // 绘制树状线条
                            let indent = draw_tree_lines(ui, depth, is_last_node, parent_is_last);
                            ui.add_space(indent);

                            // 箭头按钮
                            ui.add_space(
                                ui.spacing().button_padding.x * 2.0 + ui.spacing().icon_width,
                            );

                            // 文件名
                            let response = ui.selectable_label(is_selected, &file_name);

                            if response.clicked() {
                                let modifiers = ui.ctx().input(|i| i.modifiers);
                                let ctrl = modifiers.ctrl || modifiers.command;
                                let shift = modifiers.shift;

                                if shift && last_selected_index.is_some() {
                                    // Shift 点击：范围选择
                                    let last_idx = last_selected_index.unwrap();

                                    // 在 visible_indices 中查找位置（基于视觉顺序）
                                    let start_pos =
                                        visible_indices.iter().position(|&i| i == last_idx);
                                    let end_pos =
                                        visible_indices.iter().position(|&i| i == file_index);

                                    if let (Some(s), Some(e)) = (start_pos, end_pos) {
                                        let min_pos = s.min(e);
                                        let max_pos = s.max(e);

                                        // 清空当前选择，只选中范围内的文件
                                        selected_files.clear();
                                        for &idx in &visible_indices[min_pos..=max_pos] {
                                            selected_files.push(idx);
                                        }
                                    }
                                } else if ctrl {
                                    // Ctrl 点击：切换选中状态
                                    if is_selected {
                                        selected_files.retain(|&x| x != file_index);
                                    } else {
                                        selected_files.push(file_index);
                                    }
                                    *last_selected_index = Some(file_index);
                                } else {
                                    // 普通点击：单选
                                    selected_files.clear();
                                    selected_files.push(file_index);
                                    *last_selected_index = Some(file_index);
                                }
                            }
                        });
                    });

                    // 文件大小列
                    row.col(|ui| {
                        ui.label(crate::file_manager::format_size(file.size));
                    });

                    // 写入日期列
                    row.col(|ui| {
                        ui.label(file.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
                    });

                    // 本地列
                    row.col(|ui| {
                        let ctx = ui.ctx();
                        if file.exists {
                            ui.colored_label(
                                crate::ui::theme::local_exists_color(ctx),
                                icons::CHECK,
                            );
                        } else {
                            ui.colored_label(crate::ui::theme::muted_color(ctx), icons::CLOSE);
                        }
                    });

                    // 云端列
                    row.col(|ui| {
                        let ctx = ui.ctx();
                        if file.is_persisted {
                            ui.colored_label(
                                crate::ui::theme::cloud_exists_color(ctx),
                                icons::CHECK,
                            );
                        } else {
                            ui.colored_label(crate::ui::theme::muted_color(ctx), icons::CLOSE);
                        }
                    });

                    // 状态列（最右边）
                    row.col(|ui| {
                        if let Some(status) = sync_status {
                            let ctx = ui.ctx();
                            let i18n = crate::i18n::I18n::new(crate::i18n::Language::default());
                            let (text, color) = match status {
                                SyncStatus::Synced => {
                                    (i18n.filter_synced(), crate::ui::theme::success_color(ctx))
                                }
                                SyncStatus::LocalNewer => {
                                    (i18n.status_local_newer(), crate::ui::theme::info_color(ctx))
                                }
                                SyncStatus::CloudNewer => (
                                    i18n.status_cloud_newer(),
                                    crate::ui::theme::warning_color(ctx),
                                ),
                                SyncStatus::Conflict => {
                                    (i18n.status_conflict(), crate::ui::theme::error_color(ctx))
                                }
                                SyncStatus::LocalOnly => {
                                    (i18n.status_local_only(), crate::ui::theme::muted_color(ctx))
                                }
                                SyncStatus::CloudOnly => (
                                    i18n.status_cloud_only(),
                                    crate::ui::theme::cloud_only_color(ctx),
                                ),
                                SyncStatus::Unknown => ("?", crate::ui::theme::muted_color(ctx)),
                            };
                            ui.colored_label(color, text);
                        }
                    });
                });
            }
        }
    }
}
