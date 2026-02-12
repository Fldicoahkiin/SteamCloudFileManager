use crate::conflict::{FileComparison, SyncStatus};
use crate::i18n::I18n;
use crate::icons;
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

// 文件对比统计数据
struct ComparisonStats {
    total: usize,
    conflicts: usize,
    local_newer: usize,
    cloud_newer: usize,
    synced: usize,
}

// 过滤后的文件项
struct FilteredItem {
    index: usize,
    comparison: FileComparison,
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

    let stats = compute_statistics(&dialog.comparisons);
    let filtered = filter_comparisons(&dialog.comparisons, dialog.filter);
    let selected_comparison = dialog
        .selected_index
        .and_then(|idx| dialog.comparisons.get(idx).cloned());

    let current_filter = dialog.filter;
    let current_selected = dialog.selected_index;

    let mut new_filter = current_filter;
    let mut new_selected = current_selected;
    let mut retry_hash_filename: Option<String> = None;

    // 渲染对话框窗口
    egui::Window::new(i18n.file_comparison_title())
        .id(egui::Id::new("conflict_dialog"))
        .open(&mut dialog.show)
        .default_size([900.0, 600.0])
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            // 顶部过滤栏
            new_filter = render_filter_bar(ui, &stats, current_filter, i18n);

            ui.separator();

            // 文件列表表格
            new_selected = render_file_table(ui, &filtered, current_selected, i18n);

            ui.separator();

            // 选中文件的详细信息面板
            if let Some(comparison) = &selected_comparison {
                retry_hash_filename = render_detail_panel(ui, comparison, i18n);
            }

            // 底部状态栏
            if stats.conflicts > 0 {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(i18n.conflicts_warning(stats.conflicts))
                            .color(crate::ui::theme::error_color(ui.ctx())),
                    );
                });
            }
        });

    // 应用状态更改
    dialog.filter = new_filter;
    dialog.selected_index = new_selected;

    // 返回事件
    if let Some(filename) = retry_hash_filename {
        ConflictDialogEvent::RetryHashCheck(filename)
    } else {
        ConflictDialogEvent::None
    }
}

// 计算文件对比统计数据
fn compute_statistics(comparisons: &[FileComparison]) -> ComparisonStats {
    let mut stats = ComparisonStats {
        total: comparisons.len(),
        conflicts: 0,
        local_newer: 0,
        cloud_newer: 0,
        synced: 0,
    };

    for c in comparisons {
        match c.status {
            SyncStatus::Conflict => stats.conflicts += 1,
            SyncStatus::LocalNewer | SyncStatus::LocalOnly => stats.local_newer += 1,
            SyncStatus::CloudNewer | SyncStatus::CloudOnly => stats.cloud_newer += 1,
            SyncStatus::Synced => stats.synced += 1,
            _ => {}
        }
    }

    stats
}

// 根据过滤器筛选文件对比数据
fn filter_comparisons(
    comparisons: &[FileComparison],
    filter: SyncStatusFilter,
) -> Vec<FilteredItem> {
    comparisons
        .iter()
        .enumerate()
        .filter(|(_, c)| match filter {
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
        .map(|(idx, c)| FilteredItem {
            index: idx,
            comparison: c.clone(),
        })
        .collect()
}

// 渲染顶部过滤栏
fn render_filter_bar(
    ui: &mut egui::Ui,
    stats: &ComparisonStats,
    current_filter: SyncStatusFilter,
    i18n: &I18n,
) -> SyncStatusFilter {
    let mut new_filter = current_filter;

    ui.horizontal(|ui| {
        ui.label(i18n.total_files_count(stats.total));
        ui.separator();

        // 过滤按钮
        if ui
            .selectable_label(current_filter == SyncStatusFilter::All, i18n.filter_all())
            .clicked()
        {
            new_filter = SyncStatusFilter::All;
        }

        if stats.conflicts > 0 {
            let label = format!(
                "{} {} ({})",
                icons::WARNING,
                i18n.filter_conflicts(),
                stats.conflicts
            );
            if ui
                .selectable_label(current_filter == SyncStatusFilter::Conflicts, label)
                .clicked()
            {
                new_filter = SyncStatusFilter::Conflicts;
            }
        }

        if stats.local_newer > 0 {
            let label = format!(
                "{} {} ({})",
                icons::ARROW_UP,
                i18n.filter_local_newer(),
                stats.local_newer
            );
            if ui
                .selectable_label(current_filter == SyncStatusFilter::LocalNewer, label)
                .clicked()
            {
                new_filter = SyncStatusFilter::LocalNewer;
            }
        }

        if stats.cloud_newer > 0 {
            let label = format!(
                "{} {} ({})",
                icons::ARROW_DOWN,
                i18n.filter_cloud_newer(),
                stats.cloud_newer
            );
            if ui
                .selectable_label(current_filter == SyncStatusFilter::CloudNewer, label)
                .clicked()
            {
                new_filter = SyncStatusFilter::CloudNewer;
            }
        }

        if stats.synced > 0 {
            let label = format!(
                "{} {} ({})",
                icons::CHECK,
                i18n.filter_synced(),
                stats.synced
            );
            if ui
                .selectable_label(current_filter == SyncStatusFilter::Synced, label)
                .clicked()
            {
                new_filter = SyncStatusFilter::Synced;
            }
        }
    });

    new_filter
}

// 渲染文件列表表格
fn render_file_table(
    ui: &mut egui::Ui,
    filtered: &[FilteredItem],
    current_selected: Option<usize>,
    i18n: &I18n,
) -> Option<usize> {
    let mut new_selected = current_selected;

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

                    for item in filtered {
                        let is_selected = current_selected == Some(item.index);
                        let comparison = &item.comparison;

                        // 状态图标
                        let (status_text, status_color) =
                            get_status_display(comparison.status, ui.ctx());

                        if ui
                            .selectable_label(
                                is_selected,
                                egui::RichText::new(status_text).color(status_color),
                            )
                            .clicked()
                        {
                            new_selected = Some(item.index);
                        }

                        // 文件名
                        if ui
                            .selectable_label(is_selected, &comparison.filename)
                            .clicked()
                        {
                            new_selected = Some(item.index);
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
                        let (hash_text, hash_color) =
                            get_hash_display(comparison.hash_status, ui.ctx());
                        ui.colored_label(hash_color, hash_text);

                        ui.end_row();
                    }
                });
        });

    new_selected
}

// 渲染选中文件的详细信息面板
fn render_detail_panel(
    ui: &mut egui::Ui,
    comparison: &FileComparison,
    i18n: &I18n,
) -> Option<String> {
    let mut retry_filename: Option<String> = None;

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.strong(i18n.selected_file());
            ui.label(&comparison.filename);
        });

        ui.horizontal(|ui| {
            ui.label(i18n.status_label());
            ui.label(comparison.status_display());

            let time_diff_minutes = comparison.time_diff_secs / 60;
            if time_diff_minutes != 0 {
                let diff_text = if time_diff_minutes > 0 {
                    i18n.local_newer_by_minutes(time_diff_minutes)
                } else {
                    i18n.cloud_newer_by_minutes(-time_diff_minutes)
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
        if flags.exists_diff || flags.persisted_diff || flags.size_diff || flags.time_diff {
            ui.horizontal(|ui| {
                ui.label("差异项:");
                if flags.exists_diff {
                    ui.colored_label(crate::ui::theme::error_color(ui.ctx()), "存在");
                }
                if flags.persisted_diff {
                    ui.colored_label(crate::ui::theme::warning_color(ui.ctx()), "同步");
                }
                if flags.size_diff {
                    ui.colored_label(crate::ui::theme::warning_color(ui.ctx()), "大小");
                }
                if flags.time_diff {
                    ui.colored_label(crate::ui::theme::info_color(ui.ctx()), "时间");
                }
            });
        }

        // 显示 hash 信息
        ui.horizontal(|ui| {
            ui.label("Hash 状态:");
            ui.label(comparison.hash_status_display());
            // 重新检测按钮
            if ui
                .small_button(icons::REFRESH)
                .on_hover_text("重新检测 Hash")
                .clicked()
            {
                retry_filename = Some(comparison.filename.clone());
            }
        });

        // 分别显示本地和云端 hash
        if let Some(ref local) = comparison.local {
            ui.horizontal(|ui| {
                ui.label("本地 Hash:");
                if let Some(ref hash) = local.hash {
                    ui.monospace(hash);
                } else {
                    ui.colored_label(crate::ui::theme::muted_color(ui.ctx()), "未计算");
                }
            });
        }
        if let Some(ref cloud) = comparison.cloud {
            ui.horizontal(|ui| {
                ui.label("云端 Hash:");
                if let Some(ref hash) = cloud.hash {
                    ui.monospace(hash);
                } else {
                    ui.colored_label(crate::ui::theme::muted_color(ui.ctx()), "未计算");
                }
            });
        }
    });

    retry_filename
}

// 获取同步状态的显示文本和颜色
fn get_status_display(status: SyncStatus, ctx: &egui::Context) -> (&'static str, egui::Color32) {
    match status {
        SyncStatus::Synced => (icons::CHECK, crate::ui::theme::success_color(ctx)),
        SyncStatus::LocalNewer => (icons::ARROW_UP, crate::ui::theme::info_color(ctx)),
        SyncStatus::CloudNewer => (icons::ARROW_DOWN, crate::ui::theme::warning_color(ctx)),
        SyncStatus::Conflict => (icons::WARNING, crate::ui::theme::error_color(ctx)),
        SyncStatus::LocalOnly => (icons::FILE, crate::ui::theme::muted_color(ctx)),
        SyncStatus::CloudOnly => (icons::CLOUD, crate::ui::theme::muted_color(ctx)),
        SyncStatus::Unknown => (icons::QUESTION, crate::ui::theme::muted_color(ctx)),
    }
}

// 获取 Hash 状态的显示文本和颜色
fn get_hash_display(
    status: crate::conflict::HashStatus,
    ctx: &egui::Context,
) -> (&'static str, egui::Color32) {
    match status {
        crate::conflict::HashStatus::Pending => {
            (icons::HOURGLASS, crate::ui::theme::muted_color(ctx))
        }
        crate::conflict::HashStatus::Skipped => {
            (icons::CHECK, crate::ui::theme::success_color(ctx))
        }
        crate::conflict::HashStatus::Checking => {
            (icons::SPINNER, crate::ui::theme::warning_color(ctx))
        }
        crate::conflict::HashStatus::Match => (icons::CHECK, crate::ui::theme::success_color(ctx)),
        crate::conflict::HashStatus::Mismatch => (icons::ERROR, crate::ui::theme::error_color(ctx)),
        crate::conflict::HashStatus::Error => (icons::WARNING, crate::ui::theme::error_color(ctx)),
    }
}
