use crate::file_manager::{format_size, UploadQueue};
use egui::{Color32, RichText};

// 上传对话框的操作结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UploadAction {
    None,
    Confirm, // 确认上传
    Cancel,  // 取消
}

// 文件预览对话框
pub struct UploadPreviewDialog {
    pub queue: UploadQueue,
    pub show: bool,
}

impl UploadPreviewDialog {
    pub fn new(queue: UploadQueue) -> Self {
        Self { queue, show: true }
    }

    pub fn draw(&mut self, ctx: &egui::Context) -> UploadAction {
        let mut action = UploadAction::None;

        if !self.show {
            return action;
        }

        egui::Window::new("准备上传")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // 统计信息
                let total_files = self.queue.total_files();
                let total_size = self.queue.total_size();

                ui.label(format!("将要上传 {} 个文件到 Steam Cloud", total_files));
                ui.label(format!("总大小: {}", format_size(total_size)));

                ui.add_space(10.0);

                // 文件列表（带滚动）
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        self.draw_file_list(ui);
                    });

                ui.add_space(10.0);

                // 警告信息
                if self.has_warnings() {
                    ui.colored_label(Color32::from_rgb(255, 193, 7), "⚠️ 警告：");
                    ui.label("• 同名文件将被覆盖");
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // 操作按钮
                ui.horizontal(|ui| {
                    if ui.button("📄 添加文件").clicked() {
                        if let Some(paths) = rfd::FileDialog::new().pick_files() {
                            for path in paths {
                                if let Err(e) = self.queue.add_file(path.clone()) {
                                    tracing::warn!("添加文件失败 {}: {}", path.display(), e);
                                }
                            }
                        }
                    }

                    if ui.button("📁 添加文件夹").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            if let Err(e) = self.queue.add_folder(&folder) {
                                tracing::warn!("添加文件夹失败 {}: {}", folder.display(), e);
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✓ 确认上传").clicked() {
                            action = UploadAction::Confirm;
                            self.show = false;
                        }

                        ui.add_space(10.0);

                        if ui.button("取消").clicked() {
                            action = UploadAction::Cancel;
                            self.show = false;
                        }
                    });
                });
            });

        action
    }

    fn draw_file_list(&self, ui: &mut egui::Ui) {
        // 按文件夹分组显示
        let mut current_folder = String::new();

        for task in &self.queue.tasks {
            let path_parts: Vec<&str> = task.cloud_path.split('/').collect();

            if path_parts.len() > 1 {
                // 有文件夹
                let folder = path_parts[..path_parts.len() - 1].join("/");
                if folder != current_folder {
                    current_folder = folder.clone();
                    ui.label(RichText::new(format!("📁 {}/", folder)).strong());
                }
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label(format!(
                        "📄 {}  ({})",
                        path_parts.last().unwrap(),
                        format_size(task.size)
                    ));
                });
            } else {
                // 根目录文件
                ui.label(format!(
                    "📄 {}  ({})",
                    task.cloud_path,
                    format_size(task.size)
                ));
            }
        }
    }

    fn has_warnings(&self) -> bool {
        // TODO: 检测冲突
        false
    }
}

// 上传进度对话框
pub struct UploadProgressDialog {
    pub show: bool,
    pub current_file: String,
    pub current_index: usize,
    pub total_files: usize,
    pub progress: f32,
    pub speed: f64,
    pub completed_files: Vec<String>,
}

impl UploadProgressDialog {
    pub fn new(total_files: usize) -> Self {
        Self {
            show: true,
            current_file: String::new(),
            current_index: 0,
            total_files,
            progress: 0.0,
            speed: 0.0,
            completed_files: Vec::new(),
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        if !self.show {
            return;
        }

        egui::Window::new("📤 正在上传文件")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // 进度条（蓝色主题）
                    let progress_color = Color32::from_rgb(33, 150, 243); // #2196F3
                    ui.add(
                        egui::ProgressBar::new(self.progress)
                            .fill(progress_color)
                            .show_percentage()
                            .animate(true),
                    );

                    ui.add_space(10.0);

                    // 当前文件
                    ui.label(format!("正在上传: {}", self.current_file));
                    ui.label(format!(
                        "进度: {} / {} 文件",
                        self.current_index, self.total_files
                    ));

                    if self.speed > 0.0 {
                        ui.label(format!("速度: {}/s", format_size(self.speed as u64)));
                    }

                    ui.add_space(10.0);

                    // 已完成文件列表
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for file in &self.completed_files {
                                ui.label(format!("✓ {}", file));
                            }
                            if !self.current_file.is_empty() {
                                ui.label(format!("⏳ {}", self.current_file));
                            }
                        });

                    ui.add_space(10.0);

                    // 控制按钮
                    ui.horizontal(|ui| {
                        if ui.button("✕ 取消").clicked() {
                            self.show = false;
                        }
                    });
                });
            });
    }
}

// 上传完成对话框
pub struct UploadCompleteDialog {
    pub show: bool,
    pub success_count: usize,
    pub failed_count: usize,
    pub total_size: u64,
    pub elapsed_secs: u64,
    pub failed_files: Vec<(String, String)>,
}

impl UploadCompleteDialog {
    pub fn new(
        success_count: usize,
        failed_count: usize,
        total_size: u64,
        elapsed_secs: u64,
        failed_files: Vec<(String, String)>,
    ) -> Self {
        Self {
            show: true,
            success_count,
            failed_count,
            total_size,
            elapsed_secs,
            failed_files,
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) -> bool {
        let mut should_close = false;

        if !self.show {
            return should_close;
        }

        egui::Window::new("✓ 上传完成")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if self.failed_count == 0 {
                        ui.label(
                            RichText::new(format!("🎉 成功上传 {} 个文件", self.success_count))
                                .size(16.0)
                                .color(Color32::from_rgb(76, 175, 80)),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "⚠️ 上传完成：成功 {}，失败 {}",
                                self.success_count, self.failed_count
                            ))
                            .size(16.0)
                            .color(Color32::from_rgb(255, 193, 7)),
                        );
                    }

                    ui.add_space(10.0);

                    ui.label(format!("总大小: {}", format_size(self.total_size)));
                    ui.label(format!("用时: {} 秒", self.elapsed_secs));

                    if self.elapsed_secs > 0 {
                        let speed = self.total_size as f64 / self.elapsed_secs as f64;
                        ui.label(format!("平均速度: {}/s", format_size(speed as u64)));
                    }

                    ui.add_space(10.0);

                    // 显示失败文件列表
                    if self.failed_count > 0 {
                        ui.separator();
                        ui.label(
                            RichText::new("失败文件列表：").color(Color32::from_rgb(244, 67, 54)),
                        );

                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                for (filename, error) in &self.failed_files {
                                    ui.horizontal(|ui| {
                                        ui.label("✖");
                                        ui.label(RichText::new(filename).color(Color32::GRAY));
                                    });
                                    ui.label(
                                        RichText::new(format!("  原因: {}", error))
                                            .size(12.0)
                                            .color(Color32::DARK_GRAY),
                                    );
                                    ui.add_space(5.0);
                                }
                            });
                    }

                    ui.add_space(10.0);

                    if ui.button("确定").clicked() {
                        self.show = false;
                        should_close = true;
                    }
                });
            });

        should_close
    }
}
