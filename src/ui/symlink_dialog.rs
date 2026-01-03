use crate::i18n::I18n;
use crate::symlink_manager::{LinkDirection, LinkStatus, SymlinkConfig, SymlinkManager};
use egui::RichText;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymlinkAction {
    None,
    Close,
}

// 待执行的操作
#[derive(Default)]
struct PendingOperations {
    delete_config: Option<String>,
    create_link: Option<usize>,
    remove_link: Option<usize>,
    add_config: Option<SymlinkConfig>,
    add_and_create: Option<SymlinkConfig>,
    refresh: bool,
    copy_command: Option<usize>,
}

// 软链接管理对话框状态
pub struct SymlinkDialog {
    pub show: bool,
    pub app_id: u32,
    pub game_name: String,

    // 已配置的软链接
    configs: Vec<SymlinkConfig>,
    config_statuses: Vec<LinkStatus>,

    // 新建软链接表单
    new_direction: LinkDirection,
    new_local_path: String,
    new_remote_subfolder: String,

    // 状态消息
    status_message: Option<(String, bool)>, // (message, is_error)

    // 管理器
    manager: Option<SymlinkManager>,

    // 缓存的 remote 目录路径
    remote_dir: PathBuf,
}

impl SymlinkDialog {
    pub fn new(app_id: u32, game_name: String, steam_path: PathBuf, user_id: String) -> Self {
        let manager = SymlinkManager::new(steam_path, user_id).ok();
        let remote_dir = manager
            .as_ref()
            .map(|m| m.get_remote_dir(app_id))
            .unwrap_or_default();

        let mut dialog = Self {
            show: true,
            app_id,
            game_name,
            configs: Vec::new(),
            config_statuses: Vec::new(),
            new_direction: LinkDirection::RemoteToLocal,
            new_local_path: String::new(),
            new_remote_subfolder: String::new(),
            status_message: None,
            manager,
            remote_dir,
        };

        dialog.refresh_configs();
        dialog
    }

    fn refresh_configs(&mut self) {
        if let Some(manager) = &self.manager {
            match manager.get_configs_for_app(self.app_id) {
                Ok(configs) => {
                    self.config_statuses =
                        configs.iter().map(|c| manager.verify_symlink(c)).collect();
                    self.configs = configs;
                }
                Err(e) => {
                    tracing::warn!("加载软链接配置失败: {}", e);
                    self.configs = Vec::new();
                    self.config_statuses = Vec::new();
                }
            }
        }
    }

    fn execute_pending(&mut self, ops: PendingOperations, i18n: &I18n) {
        let mut need_refresh = false;
        let mut message: Option<(String, bool)> = None;

        // 删除配置
        if let Some(id) = ops.delete_config {
            if let Some(manager) = &self.manager {
                if let Err(e) = manager.remove_config(&id) {
                    message = Some((format!("删除配置失败: {}", e), true));
                } else {
                    need_refresh = true;
                    message = Some((i18n.symlink_config_deleted().to_string(), false));
                }
            }
        }

        // 创建链接
        if let Some(i) = ops.create_link {
            if let (Some(manager), Some(config)) = (&self.manager, self.configs.get(i)) {
                if let Err(e) = manager.create_symlink(config) {
                    message = Some((format!("{}: {}", i18n.symlink_create_failed(), e), true));
                } else {
                    need_refresh = true;
                    message = Some((i18n.symlink_created().to_string(), false));
                }
            }
        }

        // 删除链接
        if let Some(i) = ops.remove_link {
            if let (Some(manager), Some(config)) = (&self.manager, self.configs.get(i)) {
                if let Err(e) = manager.remove_symlink(config) {
                    message = Some((format!("{}: {}", i18n.symlink_remove_failed(), e), true));
                } else {
                    need_refresh = true;
                    message = Some((i18n.symlink_removed().to_string(), false));
                }
            }
        }

        // 添加配置
        if let Some(config) = ops.add_config {
            if let Some(manager) = &self.manager {
                if let Err(e) = manager.add_config(config) {
                    message = Some((format!("{}: {}", i18n.symlink_add_failed(), e), true));
                } else {
                    self.new_local_path.clear();
                    self.new_remote_subfolder.clear();
                    need_refresh = true;
                    message = Some((i18n.symlink_config_added().to_string(), false));
                }
            }
        }

        // 添加并创建
        if let Some(config) = ops.add_and_create {
            if let Some(manager) = &self.manager {
                if let Err(e) = manager.add_config(config.clone()) {
                    message = Some((format!("{}: {}", i18n.symlink_add_failed(), e), true));
                } else {
                    match manager.create_symlink(&config) {
                        Ok(_) => {
                            message = Some((i18n.symlink_created().to_string(), false));
                        }
                        Err(e) => {
                            message =
                                Some((format!("{}: {}", i18n.symlink_create_failed(), e), true));
                        }
                    }
                    self.new_local_path.clear();
                    self.new_remote_subfolder.clear();
                    need_refresh = true;
                }
            }
        }

        // 刷新
        if ops.refresh {
            need_refresh = true;
        }

        // 设置状态消息
        if let Some(msg) = message {
            self.status_message = Some(msg);
        }

        // 刷新配置 - 在所有 manager 借用结束后
        if need_refresh {
            self.refresh_configs();
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context, i18n: &I18n) -> SymlinkAction {
        let mut action = SymlinkAction::None;

        if !self.show {
            return action;
        }

        if self.manager.is_none() {
            self.show = false;
            return action;
        }

        let remote_dir = self.remote_dir.clone();
        let mut pending = PendingOperations::default();
        let mut commands_to_copy: Option<Vec<String>> = None;

        egui::Window::new(i18n.symlink_title())
            .resizable(true)
            .collapsible(false)
            .min_width(600.0)
            .default_size([650.0, 500.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // 游戏信息
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.game_name).strong().size(16.0));
                    ui.label(format!("({})", self.app_id));
                });

                ui.add_space(4.0);

                // 实验性功能警告
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(255, 243, 205))
                    .inner_margin(8.0)
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(i18n.symlink_experimental_warning())
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(133, 100, 4)),
                            );
                        });
                    });

                ui.add_space(8.0);

                // Remote 目录路径
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Remote 目录:").strong());
                    ui.label(
                        RichText::new(remote_dir.to_string_lossy())
                            .size(11.0)
                            .color(crate::ui::theme::muted_color(ctx)),
                    );
                    if ui.small_button("📋").on_hover_text("复制路径").clicked() {
                        ctx.copy_text(remote_dir.to_string_lossy().to_string());
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // 已配置的软链接列表
                ui.label(RichText::new(i18n.symlink_configured_links()).strong());
                ui.add_space(4.0);

                if self.configs.is_empty() {
                    ui.label(
                        RichText::new(i18n.symlink_no_configs())
                            .color(crate::ui::theme::muted_color(ctx)),
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(180.0)
                        .show(ui, |ui| {
                            for (i, config) in self.configs.iter().enumerate() {
                                let status = &self.config_statuses[i];

                                ui.horizontal(|ui| {
                                    // 状态图标，hover 显示状态描述
                                    ui.label(RichText::new(status.icon()))
                                        .on_hover_text(status.description());

                                    // 方向图标，hover 显示方向描述
                                    let direction_icon = match config.direction {
                                        LinkDirection::RemoteToLocal => "⬆",
                                        LinkDirection::LocalToRemote => "⬇",
                                    };
                                    ui.label(direction_icon)
                                        .on_hover_text(config.direction.description());

                                    // remote 子目录
                                    ui.label(RichText::new(&config.remote_subfolder).strong());
                                    ui.label("↔");

                                    // 本地路径
                                    ui.label(
                                        RichText::new(config.local_path.to_string_lossy())
                                            .size(11.0),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // 删除配置按钮
                                            if ui
                                                .small_button("🗑")
                                                .on_hover_text(i18n.symlink_delete_config())
                                                .clicked()
                                            {
                                                pending.delete_config = Some(config.id.clone());
                                            }

                                            // 根据状态显示不同按钮
                                            match status {
                                                LinkStatus::NotExists => {
                                                    if ui
                                                        .small_button("🔗")
                                                        .on_hover_text(i18n.symlink_create())
                                                        .clicked()
                                                    {
                                                        pending.create_link = Some(i);
                                                    }
                                                }
                                                LinkStatus::Valid | LinkStatus::Broken => {
                                                    if ui
                                                        .small_button("✂")
                                                        .on_hover_text(i18n.symlink_remove_link())
                                                        .clicked()
                                                    {
                                                        pending.remove_link = Some(i);
                                                    }
                                                }
                                                LinkStatus::Conflict => {
                                                    ui.label(
                                                        RichText::new("冲突").size(10.0).color(
                                                            crate::ui::theme::error_color(ctx),
                                                        ),
                                                    );
                                                }
                                            }

                                            // 复制命令按钮
                                            if ui
                                                .small_button("📋")
                                                .on_hover_text(i18n.symlink_copy_command())
                                                .clicked()
                                            {
                                                pending.copy_command = Some(i);
                                            }
                                        },
                                    );
                                });

                                ui.add_space(2.0);
                            }
                        });
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // 添加新软链接
                ui.label(RichText::new(i18n.symlink_add_new()).strong());
                ui.add_space(8.0);

                // 方向选择
                ui.horizontal(|ui| {
                    ui.label(i18n.symlink_direction());
                    ui.selectable_value(
                        &mut self.new_direction,
                        LinkDirection::RemoteToLocal,
                        format!("⬆ {}", LinkDirection::RemoteToLocal.description()),
                    );
                    ui.selectable_value(
                        &mut self.new_direction,
                        LinkDirection::LocalToRemote,
                        format!("⬇ {}", LinkDirection::LocalToRemote.description()),
                    );
                });

                ui.add_space(4.0);

                // 本地路径
                ui.horizontal(|ui| {
                    ui.label(i18n.symlink_local_path());
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_local_path)
                            .desired_width(350.0)
                            .hint_text("/path/to/saves"),
                    );
                    if ui
                        .button("📁")
                        .on_hover_text(i18n.symlink_browse())
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.new_local_path = path.to_string_lossy().to_string();
                        }
                    }
                });

                ui.add_space(4.0);

                // Remote 子目录名
                ui.horizontal(|ui| {
                    ui.label(i18n.symlink_remote_subfolder());
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_remote_subfolder)
                            .desired_width(200.0)
                            .hint_text("MySaves"),
                    );
                });

                ui.add_space(8.0);

                // 添加按钮
                ui.horizontal(|ui| {
                    let can_add =
                        !self.new_local_path.is_empty() && !self.new_remote_subfolder.is_empty();

                    if ui
                        .add_enabled(can_add, egui::Button::new(i18n.symlink_add_config()))
                        .clicked()
                    {
                        pending.add_config = Some(SymlinkConfig::new(
                            self.app_id,
                            self.new_direction,
                            PathBuf::from(&self.new_local_path),
                            self.new_remote_subfolder.clone(),
                        ));
                    }

                    if ui
                        .add_enabled(can_add, egui::Button::new(i18n.symlink_add_and_create()))
                        .clicked()
                    {
                        pending.add_and_create = Some(SymlinkConfig::new(
                            self.app_id,
                            self.new_direction,
                            PathBuf::from(&self.new_local_path),
                            self.new_remote_subfolder.clone(),
                        ));
                    }
                });

                // 状态消息
                if let Some((msg, is_error)) = &self.status_message {
                    ui.add_space(8.0);
                    let color = if *is_error {
                        crate::ui::theme::error_color(ctx)
                    } else {
                        crate::ui::theme::success_color(ctx)
                    };
                    ui.label(RichText::new(msg).color(color));
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // 底部按钮
                ui.horizontal(|ui| {
                    if ui
                        .button("🔄")
                        .on_hover_text(i18n.symlink_refresh())
                        .clicked()
                    {
                        pending.refresh = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(i18n.close()).clicked() {
                            action = SymlinkAction::Close;
                            self.show = false;
                        }
                    });
                });
            });

        // 处理复制命令
        if let Some(i) = pending.copy_command {
            if let Some(manager) = &self.manager {
                if let Some(config) = self.configs.get(i) {
                    commands_to_copy = Some(manager.generate_commands(config));
                }
            }
        }

        if let Some(commands) = commands_to_copy {
            ctx.copy_text(commands.join("\n"));
            self.status_message = Some((i18n.symlink_command_copied().to_string(), false));
        }

        // 执行待处理操作
        self.execute_pending(pending, i18n);

        action
    }
}
