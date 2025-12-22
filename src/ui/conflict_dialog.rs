use crate::conflict::{FileComparison, SyncStatus};
use crate::i18n::I18n;
use egui;

// 文件对比对话框
pub struct ConflictDialog {
    pub show: bool,
    pub comparisons: Vec<FileComparison>,
    pub selected_index: Option<usize>,
    pub filter: SyncStatusFilter,
}

// 同步状态过滤器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncStatusFilter {
    #[default]
    All,
    Conflicts,
    LocalNewer,
    CloudNewer,
    Synced,
}

impl ConflictDialog {
    pub fn new() -> Self {
        Self {
            show: false,
            comparisons: Vec::new(),
            selected_index: None,
            filter: SyncStatusFilter::All,
        }
    }

    // 设置对比数据
    pub fn set_comparisons(&mut self, comparisons: Vec<FileComparison>) {
        self.comparisons = comparisons;
        self.selected_index = None;
        self.show = true;
    }

    // 更新 hash 检测结果
    pub fn update_hash_result(
        &mut self,
        filename: &str,
        local_hash: Option<String>,
        cloud_hash: Option<String>,
        has_error: bool,
    ) {
        if let Some(comparison) = self.comparisons.iter_mut().find(|c| c.filename == filename) {
            // 更新本地 hash
            if let Some(ref mut local) = comparison.local {
                local.hash = local_hash.clone();
            }
            // 更新云端 hash
            if let Some(ref mut cloud) = comparison.cloud {
                cloud.hash = cloud_hash.clone();
            }

            // 更新 hash 状态
            if has_error {
                comparison.hash_status = crate::conflict::HashStatus::Error;
            } else {
                match (&local_hash, &cloud_hash) {
                    (Some(lh), Some(ch)) if lh == ch => {
                        // Hash 一致 = 内容相同，强制设为已同步
                        comparison.hash_status = crate::conflict::HashStatus::Match;
                        comparison.diff_flags.hash_diff = false;
                        comparison.status = SyncStatus::Synced;
                    }
                    (Some(_), Some(_)) => {
                        // Hash 不一致 = 内容不同，根据时间判断冲突方向
                        comparison.hash_status = crate::conflict::HashStatus::Mismatch;
                        comparison.diff_flags.hash_diff = true;
                        // 保持原状态（LocalNewer/CloudNewer），或设为 Conflict
                        if comparison.status == SyncStatus::Unknown
                            || comparison.status == SyncStatus::Synced
                        {
                            comparison.status = SyncStatus::Conflict;
                        }
                    }
                    _ => {
                        comparison.hash_status = crate::conflict::HashStatus::Error;
                    }
                }
            }
        }
    }
}

impl Default for ConflictDialog {
    fn default() -> Self {
        Self::new()
    }
}

// 对话框事件
pub enum ConflictDialogEvent {
    None,
    RetryHashCheck(String), // 重新检测指定文件的 hash
}

// 绘制文件对比对话框（只读信息展示）
pub fn draw_conflict_dialog(
    ctx: &egui::Context,
    dialog: &mut ConflictDialog,
    i18n: &I18n,
) -> ConflictDialogEvent {
    if !dialog.show {
        return ConflictDialogEvent::None;
    }
    let mut event = ConflictDialogEvent::None;

    // 预先计算统计数据
    let mut conflicts = 0usize;
    let mut local_newer = 0usize;
    let mut cloud_newer = 0usize;
    let mut synced = 0usize;

    for c in &dialog.comparisons {
        match c.status {
            SyncStatus::Conflict => conflicts += 1,
            SyncStatus::LocalNewer | SyncStatus::LocalOnly => local_newer += 1,
            SyncStatus::CloudNewer | SyncStatus::CloudOnly => cloud_newer += 1,
            SyncStatus::Synced => synced += 1,
            _ => {}
        }
    }

    let total = dialog.comparisons.len();

    // 预先计算过滤后的数据
    let filtered: Vec<(usize, FileComparison)> = dialog
        .comparisons
        .iter()
        .enumerate()
        .filter(|(_, c)| match dialog.filter {
            SyncStatusFilter::All => true,
            SyncStatusFilter::Conflicts => c.status == SyncStatus::Conflict,
            SyncStatusFilter::LocalNewer => {
                matches!(c.status, SyncStatus::LocalNewer | SyncStatus::LocalOnly)
            }
            SyncStatusFilter::CloudNewer => {
                matches!(c.status, SyncStatus::CloudNewer | SyncStatus::CloudOnly)
            }
            SyncStatusFilter::Synced => c.status == SyncStatus::Synced,
        })
        .map(|(idx, c)| (idx, c.clone()))
        .collect();

    // 获取选中的文件信息
    let selected_comparison = dialog
        .selected_index
        .and_then(|idx| dialog.comparisons.get(idx).cloned());

    let current_filter = dialog.filter;
    let current_selected = dialog.selected_index;

    let mut new_filter = current_filter;
    let mut new_selected = current_selected;
    let mut should_close = false;
    let mut retry_hash_filename: Option<String> = None;

    egui::Window::new(i18n.file_comparison_title())
        .id(egui::Id::new("conflict_dialog"))
        .default_size([900.0, 600.0])
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            // 顶部统计和过滤
            ui.horizontal(|ui| {
                ui.label(i18n.total_files_count(total));
                ui.separator();

                // 过滤按钮
                if ui
                    .selectable_label(current_filter == SyncStatusFilter::All, i18n.filter_all())
                    .clicked()
                {
                    new_filter = SyncStatusFilter::All;
                }

                if conflicts > 0 {
                    let label = format!("⚠ {} ({})", i18n.filter_conflicts(), conflicts);
                    if ui
                        .selectable_label(current_filter == SyncStatusFilter::Conflicts, label)
                        .clicked()
                    {
                        new_filter = SyncStatusFilter::Conflicts;
                    }
                }

                if local_newer > 0 {
                    let label = format!("↑ {} ({})", i18n.filter_local_newer(), local_newer);
                    if ui
                        .selectable_label(current_filter == SyncStatusFilter::LocalNewer, label)
                        .clicked()
                    {
                        new_filter = SyncStatusFilter::LocalNewer;
                    }
                }

                if cloud_newer > 0 {
                    let label = format!("↓ {} ({})", i18n.filter_cloud_newer(), cloud_newer);
                    if ui
                        .selectable_label(current_filter == SyncStatusFilter::CloudNewer, label)
                        .clicked()
                    {
                        new_filter = SyncStatusFilter::CloudNewer;
                    }
                }

                if synced > 0 {
                    let label = format!("✓ {} ({})", i18n.filter_synced(), synced);
                    if ui
                        .selectable_label(current_filter == SyncStatusFilter::Synced, label)
                        .clicked()
                    {
                        new_filter = SyncStatusFilter::Synced;
                    }
                }
            });

            ui.separator();

            // 文件列表
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    egui::Grid::new("conflict_grid")
                        .num_columns(7)
                        .striped(true)
                        .min_col_width(80.0)
                        .show(ui, |ui| {
                            // 表头
                            ui.strong(i18n.column_status());
                            ui.strong(i18n.column_filename());
                            ui.strong(i18n.column_local_size());
                            ui.strong(i18n.column_cloud_size());
                            ui.strong(i18n.column_local_time());
                            ui.strong(i18n.column_cloud_time());
                            ui.strong("Hash");
                            ui.end_row();

                            for (idx, comparison) in &filtered {
                                let is_selected = current_selected == Some(*idx);

                                // 状态图标
                                let status_text = match comparison.status {
                                    SyncStatus::Synced => "✓",
                                    SyncStatus::LocalNewer => "↑",
                                    SyncStatus::CloudNewer => "↓",
                                    SyncStatus::Conflict => "⚠",
                                    SyncStatus::LocalOnly => "📁",
                                    SyncStatus::CloudOnly => "☁",
                                    SyncStatus::Unknown => "?",
                                };

                                let status_color = match comparison.status {
                                    SyncStatus::Synced => egui::Color32::GREEN,
                                    SyncStatus::LocalNewer => egui::Color32::LIGHT_BLUE,
                                    SyncStatus::CloudNewer => egui::Color32::YELLOW,
                                    SyncStatus::Conflict => egui::Color32::RED,
                                    _ => egui::Color32::GRAY,
                                };

                                if ui
                                    .selectable_label(
                                        is_selected,
                                        egui::RichText::new(status_text).color(status_color),
                                    )
                                    .clicked()
                                {
                                    new_selected = Some(*idx);
                                }

                                // 文件名
                                if ui
                                    .selectable_label(is_selected, &comparison.filename)
                                    .clicked()
                                {
                                    new_selected = Some(*idx);
                                }

                                // 本地大小
                                let local_size = comparison
                                    .local
                                    .as_ref()
                                    .filter(|l| l.exists)
                                    .map(|l| crate::file_manager::format_size(l.size))
                                    .unwrap_or_else(|| "-".to_string());
                                ui.label(&local_size);

                                // 云端大小
                                let cloud_size = comparison
                                    .cloud
                                    .as_ref()
                                    .filter(|c| c.is_persisted)
                                    .map(|c| crate::file_manager::format_size(c.size))
                                    .unwrap_or_else(|| "-".to_string());
                                ui.label(&cloud_size);

                                // 本地时间
                                let local_time = comparison
                                    .local
                                    .as_ref()
                                    .filter(|l| l.exists)
                                    .map(|l| l.modified.format("%m-%d %H:%M").to_string())
                                    .unwrap_or_else(|| "-".to_string());
                                ui.label(&local_time);

                                // 云端时间
                                let cloud_time = comparison
                                    .cloud
                                    .as_ref()
                                    .filter(|c| c.is_persisted)
                                    .map(|c| c.timestamp.format("%m-%d %H:%M").to_string())
                                    .unwrap_or_else(|| "-".to_string());
                                ui.label(&cloud_time);

                                // Hash 状态
                                let (hash_text, hash_color) = match comparison.hash_status {
                                    crate::conflict::HashStatus::Pending => {
                                        ("⏳", egui::Color32::GRAY)
                                    }
                                    crate::conflict::HashStatus::Checking => {
                                        ("🔄", egui::Color32::YELLOW)
                                    }
                                    crate::conflict::HashStatus::Match => {
                                        ("✓", egui::Color32::GREEN)
                                    }
                                    crate::conflict::HashStatus::Mismatch => {
                                        ("✗", egui::Color32::RED)
                                    }
                                    crate::conflict::HashStatus::Error => ("⚠", egui::Color32::RED),
                                    crate::conflict::HashStatus::Skipped => {
                                        ("-", egui::Color32::GRAY)
                                    }
                                };
                                ui.colored_label(hash_color, hash_text);

                                ui.end_row();
                            }
                        });
                });

            ui.separator();

            // 选中文件的详细信息
            if let Some(comparison) = &selected_comparison {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(i18n.selected_file());
                        ui.label(&comparison.filename);
                    });

                    ui.horizontal(|ui| {
                        ui.label(i18n.status_label());
                        ui.label(comparison.status_display());

                        if comparison.time_diff_secs != 0 {
                            let diff_text = if comparison.time_diff_secs > 0 {
                                i18n.local_newer_by(comparison.time_diff_secs)
                            } else {
                                i18n.cloud_newer_by(-comparison.time_diff_secs)
                            };
                            ui.label(diff_text);
                        }
                    });

                    // 显示大小差异
                    if comparison.size_diff_bytes != 0 {
                        ui.horizontal(|ui| {
                            ui.label("大小差异:");
                            let size_text = if comparison.size_diff_bytes > 0 {
                                format!("本地大 {} bytes", comparison.size_diff_bytes)
                            } else {
                                format!("云端大 {} bytes", -comparison.size_diff_bytes)
                            };
                            ui.label(size_text);
                        });
                    }

                    // 显示各项差异标记
                    let flags = &comparison.diff_flags;
                    if flags.exists_diff
                        || flags.persisted_diff
                        || flags.size_diff
                        || flags.time_diff
                    {
                        ui.horizontal(|ui| {
                            ui.label("差异项:");
                            if flags.exists_diff {
                                ui.colored_label(egui::Color32::RED, "存在");
                            }
                            if flags.persisted_diff {
                                ui.colored_label(egui::Color32::YELLOW, "同步");
                            }
                            if flags.size_diff {
                                ui.colored_label(egui::Color32::YELLOW, "大小");
                            }
                            if flags.time_diff {
                                ui.colored_label(egui::Color32::LIGHT_BLUE, "时间");
                            }
                        });
                    }

                    // 显示 hash 信息
                    ui.horizontal(|ui| {
                        ui.label("Hash 状态:");
                        ui.label(comparison.hash_status_display());
                        // 重新检测按钮
                        if ui
                            .small_button("🔄")
                            .on_hover_text("重新检测 Hash")
                            .clicked()
                        {
                            retry_hash_filename = Some(comparison.filename.clone());
                        }
                    });

                    // 分别显示本地和云端 hash
                    if let Some(ref local) = comparison.local {
                        ui.horizontal(|ui| {
                            ui.label("本地 Hash:");
                            if let Some(ref hash) = local.hash {
                                ui.monospace(hash);
                            } else {
                                ui.colored_label(egui::Color32::GRAY, "未计算");
                            }
                        });
                    }
                    if let Some(ref cloud) = comparison.cloud {
                        ui.horizontal(|ui| {
                            ui.label("云端 Hash:");
                            if let Some(ref hash) = cloud.hash {
                                ui.monospace(hash);
                            } else {
                                ui.colored_label(egui::Color32::GRAY, "未计算");
                            }
                        });
                    }
                });
            }

            ui.separator();

            // 底部关闭按钮
            ui.horizontal(|ui| {
                if conflicts > 0 {
                    ui.label(
                        egui::RichText::new(i18n.conflicts_warning(conflicts))
                            .color(egui::Color32::RED),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(i18n.close()).clicked() {
                        should_close = true;
                    }
                });
            });
        });

    // 应用状态更改
    dialog.filter = new_filter;
    dialog.selected_index = new_selected;
    if should_close {
        dialog.show = false;
    }

    // 返回事件
    if let Some(filename) = retry_hash_filename {
        event = ConflictDialogEvent::RetryHashCheck(filename);
    }
    event
}
