use crate::config::{UfsInjectionConfig, get_ufs_injection_configs_for_app};
use crate::i18n::I18n;
use crate::icons;
use crate::path_resolver::{RootType, SaveFileConfig, get_current_platform};
use crate::vdf_parser::UfsConfig;

// 官方文档支持的所有 Root 类型
// 参考: https://partner.steamgames.com/doc/features/cloud
pub const ALL_ROOT_TYPES: &[(&str, &str, &str)] = &[
    // (名称, 平台, 描述)
    (
        "App Install Directory",
        "all",
        "[Steam]/SteamApps/common/[Game]/",
    ),
    ("SteamCloudDocuments", "all", "~/.SteamCloud/[user]/[Game]/"),
    ("WinMyDocuments", "windows", "%USERPROFILE%\\My Documents\\"),
    (
        "WinAppDataLocal",
        "windows",
        "%USERPROFILE%\\AppData\\Local\\",
    ),
    (
        "WinAppDataLocalLow",
        "windows",
        "%USERPROFILE%\\AppData\\LocalLow\\",
    ),
    (
        "WinAppDataRoaming",
        "windows",
        "%USERPROFILE%\\AppData\\Roaming\\",
    ),
    ("WinSavedGames", "windows", "%USERPROFILE%\\Saved Games\\"),
    ("MacHome", "macos", "~/"),
    ("MacAppSupport", "macos", "~/Library/Application Support/"),
    ("MacDocuments", "macos", "~/Documents/"),
    ("LinuxHome", "linux", "~/"),
    ("LinuxXdgDataHome", "linux", "$XDG_DATA_HOME/"),
];

// AppInfo 对话框状态
#[derive(Clone)]
pub struct AppInfoDialog {
    pub app_id: u32,
    pub config: UfsConfig,
    // UFS 调试功能
    pub custom_root: String,
    pub custom_path: String,
    pub custom_pattern: String,
    pub custom_platforms: Vec<String>, // 选中的平台
    pub inject_status: Option<String>,
    pub show_inject_section: bool,
    pub saved_configs: Vec<UfsInjectionConfig>, // 已保存的配置
}

impl AppInfoDialog {
    pub fn new(app_id: u32, config: UfsConfig) -> Self {
        // 加载已保存的配置
        let saved_configs = get_ufs_injection_configs_for_app(app_id);

        // 根据当前平台选择默认 Root
        let default_root = match get_current_platform() {
            "windows" => "WinAppDataLocal",
            "macos" => "MacAppSupport",
            "linux" => "LinuxHome",
            _ => "SteamCloudDocuments",
        };

        Self {
            app_id,
            config,
            custom_root: default_root.to_string(),
            custom_path: "".to_string(),
            custom_pattern: "*".to_string(),
            custom_platforms: vec!["all".to_string()], // 默认所有平台
            inject_status: None,
            show_inject_section: false,
            saved_configs,
        }
    }

    // 刷新已保存的配置
    pub fn refresh_saved_configs(&mut self) {
        self.saved_configs = get_ufs_injection_configs_for_app(self.app_id);
    }

    // 获取自定义 savefile 配置
    pub fn get_custom_savefile(&self) -> Option<SaveFileConfig> {
        if self.custom_path.is_empty() {
            return None;
        }

        Some(SaveFileConfig {
            root: self.custom_root.clone(),
            root_type: RootType::from_name(&self.custom_root),
            path: self.custom_path.clone(),
            pattern: self.custom_pattern.clone(),
            platforms: self.custom_platforms.clone(),
            recursive: true,
        })
    }

    // 获取当前平台可用的 Root 类型
    pub fn get_available_roots(&self) -> Vec<(&'static str, &'static str)> {
        let current = get_current_platform();
        ALL_ROOT_TYPES
            .iter()
            .filter(|(_, platform, _)| *platform == "all" || *platform == current)
            .map(|(name, _, desc)| (*name, *desc))
            .collect()
    }
}

// 对话框返回的动作
pub enum AppInfoDialogAction {
    None,
    Close,
    InjectUfs,
    SaveConfig,                     // 保存配置到持久化存储
    DeleteConfig(String),           // 删除指定 ID 的配置
    LoadConfig(UfsInjectionConfig), // 加载配置到输入框
    RestartSteam,
    RefreshConfig,
}

// 绘制 AppInfo 对话框
pub fn draw_appinfo_dialog(
    ctx: &egui::Context,
    dialog: &mut AppInfoDialog,
    i18n: &I18n,
) -> AppInfoDialogAction {
    let mut action = AppInfoDialogAction::None;
    let mut open = true;

    let title = i18n.appinfo_debug_title(dialog.app_id);

    egui::Window::new(title)
        .open(&mut open)
        .resizable(true)
        .default_width(600.0)
        .default_height(500.0)
        .show(ctx, |ui| {
            // 配额信息
            ui.horizontal(|ui| {
                ui.label(i18n.appinfo_quota());
                ui.label(crate::file_manager::format_size(dialog.config.quota));

                ui.separator();

                ui.label(i18n.appinfo_max_files());
                ui.label(format!("{}", dialog.config.maxnumfiles));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let refresh_label = format!("{} {}", icons::REFRESH, i18n.refresh());
                    if ui.button(refresh_label).clicked() {
                        action = AppInfoDialogAction::RefreshConfig;
                    }
                });
            });

            ui.separator();

            // UFS 配置文本
            ui.label(i18n.appinfo_current_ufs());

            egui::ScrollArea::vertical()
                .id_salt("ufs_config_scroll")
                .max_height(150.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut dialog.config.raw_text.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY),
                    );
                });

            ui.separator();

            // 自定义 UFS 注入区域（实验性）
            let inject_header = i18n.appinfo_custom_ufs();

            egui::CollapsingHeader::new(inject_header)
                .default_open(dialog.show_inject_section)
                .show(ui, |ui| {
                    dialog.show_inject_section = true;

                    ui.horizontal(|ui| {
                        ui.label(i18n.appinfo_root_type());

                        let available_roots = dialog.get_available_roots();
                        egui::ComboBox::from_id_salt("root_type_combo")
                            .selected_text(&dialog.custom_root)
                            .width(350.0)
                            .show_ui(ui, |ui| {
                                for (name, desc) in available_roots {
                                    let label = format!("{} - {}", name, desc);
                                    ui.selectable_value(
                                        &mut dialog.custom_root,
                                        name.to_string(),
                                        label,
                                    );
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label(i18n.appinfo_relative_path());
                        ui.add(
                            egui::TextEdit::singleline(&mut dialog.custom_path)
                                .hint_text(i18n.appinfo_path_hint())
                                .desired_width(300.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label(i18n.appinfo_pattern());
                        ui.add(
                            egui::TextEdit::singleline(&mut dialog.custom_pattern)
                                .hint_text(i18n.appinfo_pattern_hint())
                                .desired_width(100.0),
                        );
                    });

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        let can_inject = !dialog.custom_path.is_empty();
                        if ui
                            .add_enabled(can_inject, egui::Button::new(i18n.appinfo_inject()))
                            .clicked()
                        {
                            action = AppInfoDialogAction::InjectUfs;
                        }

                        if ui
                            .add_enabled(can_inject, egui::Button::new(i18n.appinfo_save_config()))
                            .clicked()
                        {
                            action = AppInfoDialogAction::SaveConfig;
                        }

                        if ui.button(i18n.appinfo_restart_steam()).clicked() {
                            action = AppInfoDialogAction::RestartSteam;
                        }
                    });

                    // 显示注入状态
                    if let Some(status) = &dialog.inject_status {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(status).color(
                            if status.contains("成功")
                                || status.contains("Success")
                                || status.contains("保存")
                            {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::RED
                            },
                        ));
                    }

                    ui.add_space(8.0);

                    // 已保存的配置列表（类似 Steamworks 的可视化管理）
                    ui.separator();
                    ui.label(egui::RichText::new(i18n.appinfo_saved_configs()).strong());

                    if dialog.saved_configs.is_empty() {
                        ui.label(
                            egui::RichText::new(i18n.appinfo_no_saved_configs())
                                .italics()
                                .color(egui::Color32::GRAY),
                        );
                    } else {
                        egui::ScrollArea::vertical()
                            .id_salt("saved_configs_scroll")
                            .max_height(150.0)
                            .show(ui, |ui| {
                                let configs = dialog.saved_configs.clone();
                                for config in configs {
                                    egui::Frame::group(ui.style())
                                        .inner_margin(egui::Margin::same(8))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // 配置信息
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            egui::RichText::new(&config.root)
                                                                .strong()
                                                                .color(egui::Color32::from_rgb(
                                                                    100, 180, 255,
                                                                )),
                                                        );
                                                        ui.label("/");
                                                        ui.label(&config.path);
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            egui::RichText::new(format!(
                                                                "Pattern: {}",
                                                                &config.pattern
                                                            ))
                                                            .small()
                                                            .color(egui::Color32::GRAY),
                                                        );
                                                        if !config.platforms.is_empty()
                                                            && !config
                                                                .platforms
                                                                .iter()
                                                                .any(|p| p == "all")
                                                        {
                                                            ui.label(
                                                                egui::RichText::new(format!(
                                                                    "| Platforms: {}",
                                                                    config.platforms.join(", ")
                                                                ))
                                                                .small()
                                                                .color(egui::Color32::GRAY),
                                                            );
                                                        }
                                                    });
                                                });

                                                // 操作按钮（右侧）
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        // 删除按钮
                                                        if ui
                                                            .small_button(
                                                                i18n.appinfo_delete_config(),
                                                            )
                                                            .clicked()
                                                        {
                                                            action =
                                                                AppInfoDialogAction::DeleteConfig(
                                                                    config.id.clone(),
                                                                );
                                                        }

                                                        // 加载到输入框按钮
                                                        if ui
                                                            .small_button(
                                                                i18n.appinfo_load_config(),
                                                            )
                                                            .clicked()
                                                        {
                                                            action =
                                                                AppInfoDialogAction::LoadConfig(
                                                                    config.clone(),
                                                                );
                                                        }
                                                    },
                                                );
                                            });
                                        });
                                    ui.add_space(4.0);
                                }
                            });
                    }

                    ui.add_space(4.0);

                    // 实验性功能警告
                    let warning = i18n.appinfo_warning();
                    ui.label(
                        egui::RichText::new(warning)
                            .color(egui::Color32::YELLOW)
                            .small(),
                    );
                });
        });

    if !open {
        return AppInfoDialogAction::Close;
    }

    action
}
