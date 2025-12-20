use crate::backup::{BackupProgress, BackupResult};
use crate::file_manager::format_size;
use crate::i18n::I18n;
use crate::steam_api::CloudFile;
use egui::{Color32, RichText};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackupAction {
    None,
    StartBackup,
    Cancel,
    OpenBackupDir,
}

pub struct BackupPreviewDialog {
    pub show: bool,
    pub app_id: u32,
    pub game_name: String,
    pub files: Vec<CloudFile>,
}

impl BackupPreviewDialog {
    pub fn new(app_id: u32, game_name: String, files: Vec<CloudFile>) -> Self {
        Self {
            show: true,
            app_id,
            game_name,
            files,
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context, i18n: &I18n) -> BackupAction {
        let mut action = BackupAction::None;

        if !self.show {
            return action;
        }

        egui::Window::new(i18n.backup_title())
            .resizable(true)
            .collapsible(false)
            .min_width(500.0)
            .default_size([550.0, 400.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // 游戏信息
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.game_name).strong().size(16.0));
                    ui.label(format!("({})", self.app_id));
                });

                ui.add_space(8.0);

                // 统计信息
                let total_files = self.files.len();
                let total_size: u64 = self.files.iter().map(|f| f.size).sum();
                let cdp_files = self
                    .files
                    .iter()
                    .filter(|f| f.root_description.starts_with("CDP:"))
                    .count();

                ui.horizontal(|ui| {
                    ui.label(i18n.backup_file_count(total_files));
                    ui.label(" | ");
                    ui.label(i18n.backup_total_size(&format_size(total_size)));
                });

                if cdp_files < total_files {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(i18n.backup_cdp_warning(total_files - cdp_files))
                            .color(Color32::from_rgb(255, 165, 0))
                            .size(11.0),
                    );
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // 文件列表预览
                ui.label(RichText::new(i18n.backup_file_list()).strong());
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        for file in &self.files {
                            ui.horizontal(|ui| {
                                // 状态图标
                                let has_url = file.root_description.starts_with("CDP:");
                                let icon = if has_url { "✓" } else { "⚠" };
                                let color = if has_url {
                                    Color32::from_rgb(100, 200, 100)
                                } else {
                                    Color32::from_rgb(255, 165, 0)
                                };
                                ui.label(RichText::new(icon).color(color));

                                // 文件名
                                ui.label(&file.name);

                                // 大小
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(format_size(file.size))
                                                .color(Color32::GRAY),
                                        );
                                    },
                                );
                            });
                        }
                    });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // 底部按钮
                ui.horizontal(|ui| {
                    if ui.button(i18n.backup_open_dir()).clicked() {
                        action = BackupAction::OpenBackupDir;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_backup = cdp_files > 0;

                        if ui
                            .add_enabled(
                                can_backup,
                                egui::Button::new(RichText::new(i18n.backup_start()).color(
                                    if can_backup {
                                        Color32::WHITE
                                    } else {
                                        Color32::GRAY
                                    },
                                )),
                            )
                            .clicked()
                        {
                            action = BackupAction::StartBackup;
                        }

                        if ui.button(i18n.cancel()).clicked() {
                            action = BackupAction::Cancel;
                            self.show = false;
                        }
                    });
                });
            });

        action
    }
}

pub struct BackupProgressDialog {
    pub show: bool,
    pub progress: BackupProgress,
    pub result: Option<BackupResult>,
}

impl BackupProgressDialog {
    pub fn new(total_files: usize) -> Self {
        Self {
            show: true,
            progress: BackupProgress::new(total_files),
            result: None,
        }
    }

    pub fn set_result(&mut self, result: BackupResult) {
        self.result = Some(result);
    }

    pub fn draw(&mut self, ctx: &egui::Context, i18n: &I18n) -> bool {
        let mut close = false;

        if !self.show {
            return close;
        }

        egui::Window::new(i18n.backup_progress_title())
            .resizable(false)
            .collapsible(false)
            .min_width(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if let Some(result) = &self.result {
                    // 显示结果
                    ui.add_space(8.0);

                    if result.success {
                        ui.label(
                            RichText::new(i18n.backup_complete())
                                .color(Color32::from_rgb(100, 200, 100))
                                .size(16.0),
                        );
                    } else {
                        ui.label(
                            RichText::new(i18n.backup_partial())
                                .color(Color32::from_rgb(255, 165, 0))
                                .size(16.0),
                        );
                    }

                    ui.add_space(8.0);

                    ui.label(i18n.backup_result_stats(result.success_count, result.total_files));

                    if !result.failed_files.is_empty() {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(i18n.backup_failed_files())
                                .color(Color32::from_rgb(255, 100, 100)),
                        );

                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for (name, err) in &result.failed_files {
                                    ui.label(format!("• {} - {}", name, err));
                                }
                            });
                    }

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("{}", result.backup_path.display()))
                            .size(11.0)
                            .color(Color32::GRAY),
                    );

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if ui.button(i18n.backup_open_dir()).clicked() {
                            if let Err(e) = open_path(&result.backup_path) {
                                tracing::warn!("打开目录失败: {}", e);
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(i18n.close()).clicked() {
                                close = true;
                                self.show = false;
                            }
                        });
                    });
                } else {
                    // 显示进度
                    ui.add_space(8.0);

                    ui.label(i18n.backup_in_progress());
                    ui.add_space(8.0);

                    // 进度条
                    let progress = self.progress.percent() / 100.0;
                    ui.add(egui::ProgressBar::new(progress).show_percentage());

                    ui.add_space(8.0);

                    ui.label(format!(
                        "{} / {}",
                        self.progress.completed_files, self.progress.total_files
                    ));

                    if !self.progress.current_file.is_empty() {
                        ui.label(
                            RichText::new(&self.progress.current_file)
                                .size(11.0)
                                .color(Color32::GRAY),
                        );
                    }

                    ui.add_space(8.0);
                }
            });

        close
    }
}

fn open_path(path: &std::path::Path) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(path).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }

    Ok(())
}
