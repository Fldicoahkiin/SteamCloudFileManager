use crate::config::{
    PathTransform, RootOverrideEntry, SaveFileEntry, UfsGameConfig, get_ufs_game_config,
};
use crate::i18n::I18n;
use crate::icons;
use crate::path_resolver::get_current_platform;
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

// 编辑模式
#[derive(Clone, PartialEq)]
pub enum EditMode {
    None,
    AddSavefile,
    EditSavefile(usize),
    AddOverride,
    EditOverride(usize),
}

// AppInfo 对话框状态
#[derive(Clone)]
pub struct AppInfoDialog {
    pub app_id: u32,
    pub config: UfsConfig,

    // 表格编辑状态
    pub editing_savefiles: Vec<SaveFileEntry>,
    pub editing_overrides: Vec<RootOverrideEntry>,
    pub edit_mode: EditMode,

    // 临时编辑字段
    pub temp_savefile: SaveFileEntry,
    pub temp_override: RootOverrideEntry,
    // UI 临时状态：是否使用路径转换（对应 Steamworks 的 "Replace Path" 勾选框）
    pub temp_use_path_transform: bool,

    // 状态
    pub inject_status: Option<String>,
    pub game_config: Option<UfsGameConfig>,
}

impl AppInfoDialog {
    pub fn new(app_id: u32, config: UfsConfig) -> Self {
        let game_config = get_ufs_game_config(app_id);

        // 初始化编辑表格
        let (editing_savefiles, editing_overrides) = if let Some(ref gc) = game_config {
            (gc.savefiles.clone(), gc.root_overrides.clone())
        } else {
            (Vec::new(), Vec::new())
        };

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
            editing_savefiles,
            editing_overrides,
            edit_mode: EditMode::None,
            temp_savefile: SaveFileEntry {
                root: default_root.to_string(),
                path: String::new(),
                pattern: "*".to_string(),
                platforms: vec!["all".to_string()],
                recursive: true,
            },
            temp_override: RootOverrideEntry {
                original_root: "WinAppDataLocal".to_string(),
                os: "macos".to_string(),
                new_root: "MacAppSupport".to_string(),
                add_path: String::new(),
                path_transforms: Vec::new(),
            },
            temp_use_path_transform: false,
            inject_status: None,
            game_config,
        }
    }

    // 刷新已保存的配置
    pub fn refresh_saved_configs(&mut self) {
        self.game_config = get_ufs_game_config(self.app_id);
        if let Some(ref gc) = self.game_config {
            self.editing_savefiles = gc.savefiles.clone();
            self.editing_overrides = gc.root_overrides.clone();
        }
    }

    // 从 VDF 配置加载到编辑界面
    // 将 VDF 中的 SaveFileConfig 和 RootOverrideConfig 转换为可编辑的 Entry 格式
    pub fn load_from_vdf(&mut self) {
        // 转换 savefiles
        self.editing_savefiles = self
            .config
            .savefiles
            .iter()
            .map(|sf| SaveFileEntry {
                root: sf.root.clone(),
                path: sf.path.clone(),
                pattern: sf.pattern.clone(),
                platforms: if sf.platforms.is_empty() {
                    vec!["all".to_string()]
                } else {
                    sf.platforms.clone()
                },
                recursive: sf.recursive,
            })
            .collect();

        // 转换 rootoverrides
        self.editing_overrides = self
            .config
            .rootoverrides
            .iter()
            .map(|ro| RootOverrideEntry {
                original_root: ro.original_root.clone(),
                os: ro.oslist.first().cloned().unwrap_or_default(),
                new_root: ro.new_root.clone(),
                add_path: ro.add_path.clone(),
                path_transforms: ro
                    .path_transforms
                    .iter()
                    .map(|t| PathTransform {
                        find: t.find.clone(),
                        replace: t.replace.clone(),
                    })
                    .collect(),
            })
            .collect();
    }

    // 构建当前编辑的 UfsGameConfig
    pub fn build_game_config(&self) -> UfsGameConfig {
        UfsGameConfig {
            id: self
                .game_config
                .as_ref()
                .map(|c| c.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            app_id: self.app_id,
            savefiles: self.editing_savefiles.clone(),
            root_overrides: self.editing_overrides.clone(),
            created_at: self
                .game_config
                .as_ref()
                .map(|c| c.created_at)
                .unwrap_or_else(|| chrono::Utc::now().timestamp()),
            note: String::new(),
        }
    }

    // 获取所有 Root 类型（用于 Root Overrides 配置）
    pub fn get_all_roots() -> Vec<(&'static str, &'static str)> {
        ALL_ROOT_TYPES
            .iter()
            .map(|(name, _, desc)| (*name, *desc))
            .collect()
    }
}

// 对话框返回的动作
pub enum AppInfoDialogAction {
    None,
    Close,
    InjectFullConfig, // 注入完整配置到 VDF
    SaveGameConfig,   // 保存配置到文件
    ClearGameConfig,  // 清空所有自定义配置
    LoadFromVdf,      // 从 VDF 加载现有配置
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

            // 可编辑配置表格（类似 Steamworks）
            egui::CollapsingHeader::new(format!(
                "{} {}",
                icons::FOLDER,
                i18n.ufs_savefiles_header(dialog.editing_savefiles.len())
            ))
            .default_open(true)
            .show(ui, |ui| {
                // 添加按钮
                ui.horizontal(|ui| {
                    if ui
                        .button(format!("{} {}", icons::ADD_FILE, i18n.ufs_add_savefile()))
                        .clicked()
                    {
                        dialog.edit_mode = EditMode::AddSavefile;
                        // 保持 temp_savefile 的默认值（在 new() 中已初始化）
                        dialog.temp_savefile.path.clear();
                        dialog.temp_savefile.pattern = "*".to_string();
                        dialog.temp_savefile.platforms = vec!["all".to_string()];
                        dialog.temp_savefile.recursive = true;
                    }
                });

                // 添加/编辑表单
                if matches!(
                    dialog.edit_mode,
                    EditMode::AddSavefile | EditMode::EditSavefile(_)
                ) {
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", i18n.ufs_label_root()));
                                egui::ComboBox::from_id_salt("edit_savefile_root")
                                    .selected_text(&dialog.temp_savefile.root)
                                    .width(200.0)
                                    .show_ui(ui, |ui| {
                                        // 显示所有平台的 Root，跨平台映射通过 overrides 处理
                                        for (name, desc) in AppInfoDialog::get_all_roots() {
                                            ui.selectable_value(
                                                &mut dialog.temp_savefile.root,
                                                name.to_string(),
                                                format!("{} - {}", name, desc),
                                            );
                                        }
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", i18n.ufs_label_path()));
                                ui.add(
                                    egui::TextEdit::singleline(&mut dialog.temp_savefile.path)
                                        .hint_text(i18n.appinfo_path_hint())
                                        .desired_width(200.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", i18n.ufs_label_pattern()));
                                ui.add(
                                    egui::TextEdit::singleline(&mut dialog.temp_savefile.pattern)
                                        .hint_text(i18n.appinfo_pattern_hint())
                                        .desired_width(80.0),
                                );
                            });

                            ui.horizontal(|ui| {
                                if ui.button(icons::CHECK).clicked() {
                                    match dialog.edit_mode {
                                        EditMode::AddSavefile => {
                                            dialog
                                                .editing_savefiles
                                                .push(dialog.temp_savefile.clone());
                                        }
                                        EditMode::EditSavefile(idx) => {
                                            if idx < dialog.editing_savefiles.len() {
                                                dialog.editing_savefiles[idx] =
                                                    dialog.temp_savefile.clone();
                                            }
                                        }
                                        _ => {}
                                    }
                                    dialog.edit_mode = EditMode::None;
                                }
                                if ui.button(icons::CLOSE).clicked() {
                                    dialog.edit_mode = EditMode::None;
                                }
                            });
                        });
                }

                // 表格显示
                if dialog.editing_savefiles.is_empty() {
                    ui.label(
                        egui::RichText::new(i18n.ufs_no_savefiles())
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("editing_savefiles_table")
                        .max_height(120.0)
                        .show(ui, |ui| {
                            let mut to_delete: Option<usize> = None;
                            let mut to_edit: Option<usize> = None;

                            egui::Grid::new("editing_savefiles_grid")
                                .num_columns(5)
                                .striped(true)
                                .min_col_width(50.0)
                                .show(ui, |ui| {
                                    // 表头
                                    ui.label(egui::RichText::new(i18n.ufs_label_root()).strong());
                                    ui.label(egui::RichText::new(i18n.ufs_label_path()).strong());
                                    ui.label(
                                        egui::RichText::new(i18n.ufs_label_pattern()).strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(i18n.ufs_label_platforms()).strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(i18n.ufs_label_actions()).strong(),
                                    );
                                    ui.end_row();

                                    // 数据行
                                    for (idx, sf) in dialog.editing_savefiles.iter().enumerate() {
                                        ui.label(
                                            egui::RichText::new(&sf.root)
                                                .color(egui::Color32::from_rgb(100, 180, 255)),
                                        );
                                        ui.label(&sf.path);
                                        ui.label(&sf.pattern);
                                        ui.label(sf.platforms.join(", "));
                                        ui.horizontal(|ui| {
                                            if ui.small_button(icons::GEAR).clicked() {
                                                to_edit = Some(idx);
                                            }
                                            if ui.small_button(icons::TRASH).clicked() {
                                                to_delete = Some(idx);
                                            }
                                        });
                                        ui.end_row();
                                    }
                                });

                            // 处理删除
                            if let Some(idx) = to_delete {
                                dialog.editing_savefiles.remove(idx);
                            }
                            // 处理编辑
                            if let Some(idx) = to_edit {
                                dialog.temp_savefile = dialog.editing_savefiles[idx].clone();
                                dialog.edit_mode = EditMode::EditSavefile(idx);
                            }
                        });
                }
            });

            // Root Overrides 可编辑表格
            egui::CollapsingHeader::new(format!(
                "{} {}",
                icons::ARROW_SYNC,
                i18n.ufs_overrides_header(dialog.editing_overrides.len())
            ))
            .default_open(true)
            .show(ui, |ui| {
                // 添加按钮
                ui.horizontal(|ui| {
                    if ui
                        .button(format!("{} {}", icons::ADD_FILE, i18n.ufs_add_override()))
                        .clicked()
                    {
                        dialog.edit_mode = EditMode::AddOverride;
                        dialog.temp_override = RootOverrideEntry {
                            original_root: "WinAppDataLocal".to_string(),
                            os: "macos".to_string(),
                            new_root: "MacAppSupport".to_string(),
                            add_path: String::new(),
                            path_transforms: Vec::new(),
                        };
                        dialog.temp_use_path_transform = false;
                    }
                });

                // 添加/编辑表单
                if matches!(
                    dialog.edit_mode,
                    EditMode::AddOverride | EditMode::EditOverride(_)
                ) {
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", i18n.ufs_label_original_root()));
                                egui::ComboBox::from_id_salt("edit_override_original")
                                    .selected_text(&dialog.temp_override.original_root)
                                    .width(130.0)
                                    .show_ui(ui, |ui| {
                                        for (name, _) in AppInfoDialog::get_all_roots() {
                                            ui.selectable_value(
                                                &mut dialog.temp_override.original_root,
                                                name.to_string(),
                                                name,
                                            );
                                        }
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", i18n.ufs_label_target_os()));
                                egui::ComboBox::from_id_salt("edit_override_os")
                                    .selected_text(&dialog.temp_override.os)
                                    .width(80.0)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut dialog.temp_override.os,
                                            "windows".to_string(),
                                            "Windows",
                                        );
                                        ui.selectable_value(
                                            &mut dialog.temp_override.os,
                                            "macos".to_string(),
                                            "macOS",
                                        );
                                        ui.selectable_value(
                                            &mut dialog.temp_override.os,
                                            "linux".to_string(),
                                            "Linux",
                                        );
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", i18n.ufs_label_new_root()));
                                egui::ComboBox::from_id_salt("edit_override_new")
                                    .selected_text(&dialog.temp_override.new_root)
                                    .width(130.0)
                                    .show_ui(ui, |ui| {
                                        for (name, _) in AppInfoDialog::get_all_roots() {
                                            ui.selectable_value(
                                                &mut dialog.temp_override.new_root,
                                                name.to_string(),
                                                name,
                                            );
                                        }
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", i18n.ufs_label_add_path()));
                                ui.add(
                                    egui::TextEdit::singleline(&mut dialog.temp_override.add_path)
                                        .hint_text("optional")
                                        .desired_width(120.0),
                                );
                                ui.checkbox(
                                    &mut dialog.temp_use_path_transform,
                                    i18n.ufs_label_replace_path(),
                                );
                            });

                            ui.horizontal(|ui| {
                                if ui.button(icons::CHECK).clicked() {
                                    // 根据 UI 状态准备 override 条目
                                    let mut override_entry = dialog.temp_override.clone();

                                    if dialog.temp_use_path_transform {
                                        // 勾选了 "Replace Path"：使用 pathtransforms
                                        // find="" 表示匹配所有，replace=add_path 表示替换为该路径
                                        override_entry.path_transforms = vec![PathTransform {
                                            find: String::new(),
                                            replace: override_entry.add_path.clone(),
                                        }];
                                        // pathtransforms 和 addpath 互斥，清空 add_path
                                        override_entry.add_path = String::new();
                                    } else {
                                        // 未勾选：使用 addpath，清空 path_transforms
                                        override_entry.path_transforms = Vec::new();
                                    }

                                    match dialog.edit_mode {
                                        EditMode::AddOverride => {
                                            dialog.editing_overrides.push(override_entry);
                                        }
                                        EditMode::EditOverride(idx) => {
                                            if idx < dialog.editing_overrides.len() {
                                                dialog.editing_overrides[idx] = override_entry;
                                            }
                                        }
                                        _ => {}
                                    }
                                    dialog.edit_mode = EditMode::None;
                                }
                                if ui.button(icons::CLOSE).clicked() {
                                    dialog.edit_mode = EditMode::None;
                                }
                            });
                        });
                }

                // 表格显示
                if dialog.editing_overrides.is_empty() {
                    ui.label(
                        egui::RichText::new(i18n.ufs_no_overrides())
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("editing_overrides_table")
                        .max_height(100.0)
                        .show(ui, |ui| {
                            let mut to_delete: Option<usize> = None;
                            let mut to_edit: Option<usize> = None;

                            egui::Grid::new("editing_overrides_grid")
                                .num_columns(6)
                                .striped(true)
                                .min_col_width(40.0)
                                .show(ui, |ui| {
                                    // 表头
                                    ui.label(
                                        egui::RichText::new(i18n.ufs_label_original_root())
                                            .strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(i18n.ufs_label_target_os()).strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(i18n.ufs_label_new_root()).strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(i18n.ufs_label_add_path()).strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(i18n.ufs_label_replace_path()).strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(i18n.ufs_label_actions()).strong(),
                                    );
                                    ui.end_row();

                                    // 数据行
                                    for (idx, ro) in dialog.editing_overrides.iter().enumerate() {
                                        ui.label(
                                            egui::RichText::new(&ro.original_root)
                                                .color(egui::Color32::from_rgb(255, 180, 100)),
                                        );
                                        ui.label(&ro.os);
                                        ui.label(&ro.new_root);
                                        // 显示 add_path 或 pathtransforms 中的 replace 值
                                        let display_path = if !ro.path_transforms.is_empty() {
                                            ro.path_transforms
                                                .first()
                                                .map(|t| t.replace.as_str())
                                                .unwrap_or("-")
                                        } else if !ro.add_path.is_empty() {
                                            &ro.add_path
                                        } else {
                                            "-"
                                        };
                                        ui.label(display_path);
                                        // 有 pathtransforms 表示勾选了 Replace Path
                                        ui.label(if !ro.path_transforms.is_empty() {
                                            "✓"
                                        } else {
                                            "-"
                                        });
                                        ui.horizontal(|ui| {
                                            if ui.small_button(icons::GEAR).clicked() {
                                                to_edit = Some(idx);
                                            }
                                            if ui.small_button(icons::TRASH).clicked() {
                                                to_delete = Some(idx);
                                            }
                                        });
                                        ui.end_row();
                                    }
                                });

                            // 处理删除
                            if let Some(idx) = to_delete {
                                dialog.editing_overrides.remove(idx);
                            }
                            // 处理编辑
                            if let Some(idx) = to_edit {
                                let entry = &dialog.editing_overrides[idx];
                                dialog.temp_override = entry.clone();
                                // 恢复 UI 状态
                                dialog.temp_use_path_transform = !entry.path_transforms.is_empty();
                                // 如果使用 pathtransforms，将 replace 值恢复到 add_path 以便 UI 编辑
                                if dialog.temp_use_path_transform {
                                    dialog.temp_override.add_path = entry
                                        .path_transforms
                                        .first()
                                        .map(|t| t.replace.clone())
                                        .unwrap_or_default();
                                }
                                dialog.edit_mode = EditMode::EditOverride(idx);
                            }
                        });
                }
            });

            // 保存和注入按钮
            ui.separator();
            ui.horizontal(|ui| {
                let has_changes =
                    !dialog.editing_savefiles.is_empty() || !dialog.editing_overrides.is_empty();

                if ui
                    .add_enabled(has_changes, egui::Button::new(i18n.ufs_save_config()))
                    .clicked()
                {
                    action = AppInfoDialogAction::SaveGameConfig;
                }
                if ui
                    .add_enabled(has_changes, egui::Button::new(i18n.ufs_inject_to_vdf()))
                    .clicked()
                {
                    action = AppInfoDialogAction::InjectFullConfig;
                }

                // Load from VDF 按钮：当 VDF 中有配置时可用
                let has_vdf_config =
                    !dialog.config.savefiles.is_empty() || !dialog.config.rootoverrides.is_empty();
                if ui
                    .add_enabled(has_vdf_config, egui::Button::new(i18n.ufs_load_from_vdf()))
                    .on_hover_text(i18n.ufs_load_from_vdf_tooltip())
                    .clicked()
                {
                    action = AppInfoDialogAction::LoadFromVdf;
                }

                // 清空按钮：当有内容时可以点击
                let can_clear =
                    !dialog.editing_savefiles.is_empty() || !dialog.editing_overrides.is_empty();
                if ui
                    .add_enabled(
                        can_clear,
                        egui::Button::new(format!("{} {}", icons::TRASH, i18n.ufs_clear_all()))
                            .fill(egui::Color32::from_rgb(120, 40, 40)),
                    )
                    .on_hover_text(i18n.ufs_clear_all_tooltip())
                    .clicked()
                {
                    // 清空本地编辑状态
                    dialog.editing_savefiles.clear();
                    dialog.editing_overrides.clear();
                    action = AppInfoDialogAction::ClearGameConfig;
                }
            });

            ui.separator();

            // 操作提示区域
            ui.separator();
            ui.horizontal(|ui| {
                // 重启 Steam 按钮
                if ui.button(i18n.appinfo_restart_steam()).clicked() {
                    action = AppInfoDialogAction::RestartSteam;
                }

                // 刷新配置按钮
                if ui
                    .button(format!("{} {}", icons::ARROW_SYNC, i18n.ufs_refresh()))
                    .clicked()
                {
                    action = AppInfoDialogAction::RefreshConfig;
                }
            });

            // 显示注入状态
            if let Some(status) = &dialog.inject_status {
                ui.add_space(4.0);
                let color = if status.contains("成功")
                    || status.contains("Success")
                    || status.contains("Saved")
                    || status.contains("Cleared")
                {
                    egui::Color32::GREEN
                } else if status.contains("error") || status.contains("Error") {
                    egui::Color32::RED
                } else {
                    egui::Color32::YELLOW
                };
                ui.label(egui::RichText::new(status).color(color));
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

    if !open {
        return AppInfoDialogAction::Close;
    }

    action
}
