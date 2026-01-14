use crate::i18n::I18n;
use crate::icons;
use egui;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SettingsTab {
    #[default]
    Log,
    Appearance,
    Advanced,
    Backup,
    About,
}

pub struct SettingsWindowState {
    pub tab: SettingsTab,
    pub about_icon_texture: Option<egui::TextureHandle>,
    pub theme_mode: crate::ui::theme::ThemeMode,
    pub steam_path_input: String,
    pub steam_path_changed: bool,
    pub show_reset_confirm: bool,
    pub log_dir_display: String,
    pub config_path_display: String,
    pub backup_dir_display: String,
}

impl Default for SettingsWindowState {
    fn default() -> Self {
        // 获取当前 Steam 路径显示
        let current_path = crate::config::get_custom_steam_path()
            .or_else(|| crate::vdf_parser::VdfParser::find_steam_path().ok())
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        // 获取只读路径
        let log_dir = crate::logger::get_log_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let config_path = crate::config::get_config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let backup_dir = crate::backup::get_backup_root_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        Self {
            tab: SettingsTab::Log,
            about_icon_texture: None,
            theme_mode: crate::ui::theme::ThemeMode::default(),
            steam_path_input: current_path,
            steam_path_changed: false,
            show_reset_confirm: false,
            log_dir_display: log_dir,
            config_path_display: config_path,
            backup_dir_display: backup_dir,
        }
    }
}

// 绘制设置窗口
pub fn draw_settings_window(
    ctx: &egui::Context,
    show: &mut bool,
    state: &mut SettingsWindowState,
    update_manager: &mut crate::update::UpdateManager,
    i18n: &I18n,
) -> Option<crate::update::ReleaseInfo> {
    let mut download_release = None;

    let accent_color = crate::ui::theme::accent_color(ctx);

    egui::Window::new(i18n.settings_title())
        .open(show)
        .resizable(true)
        .collapsible(false)
        .default_size([520.0, 460.0])
        .min_size([450.0, 380.0])
        .show(ctx, |ui| {
            let content_height = ui.available_height().max(400.0);

            ui.horizontal(|ui| {
                // 左侧边栏
                ui.vertical(|ui| {
                    ui.set_width(80.0);
                    ui.set_min_height(content_height);
                    ui.add_space(8.0);

                    // 日志
                    let log_selected = state.tab == SettingsTab::Log;
                    let log_response = ui.add_sized(
                        [ui.available_width(), 28.0],
                        egui::Button::new(egui::RichText::new(i18n.settings_log()).color(
                            if log_selected {
                                accent_color
                            } else {
                                ui.style().visuals.text_color()
                            },
                        ))
                        .fill(if log_selected {
                            ui.style().visuals.selection.bg_fill
                        } else {
                            crate::ui::theme::transparent_color()
                        }),
                    );
                    if log_response.clicked() {
                        state.tab = SettingsTab::Log;
                    }

                    ui.add_space(4.0);

                    // 备份
                    let backup_selected = state.tab == SettingsTab::Backup;
                    let backup_response = ui.add_sized(
                        [ui.available_width(), 28.0],
                        egui::Button::new(egui::RichText::new(i18n.backup()).color(
                            if backup_selected {
                                accent_color
                            } else {
                                ui.style().visuals.text_color()
                            },
                        ))
                        .fill(if backup_selected {
                            ui.style().visuals.selection.bg_fill
                        } else {
                            crate::ui::theme::transparent_color()
                        }),
                    );
                    if backup_response.clicked() {
                        state.tab = SettingsTab::Backup;
                    }

                    ui.add_space(4.0);

                    // 外观
                    let appearance_selected = state.tab == SettingsTab::Appearance;
                    let appearance_response = ui.add_sized(
                        [ui.available_width(), 28.0],
                        egui::Button::new(egui::RichText::new(i18n.settings_appearance()).color(
                            if appearance_selected {
                                accent_color
                            } else {
                                ui.style().visuals.text_color()
                            },
                        ))
                        .fill(if appearance_selected {
                            ui.style().visuals.selection.bg_fill
                        } else {
                            crate::ui::theme::transparent_color()
                        }),
                    );
                    if appearance_response.clicked() {
                        state.tab = SettingsTab::Appearance;
                    }

                    ui.add_space(4.0);

                    // 高级
                    let advanced_selected = state.tab == SettingsTab::Advanced;
                    let advanced_response = ui.add_sized(
                        [ui.available_width(), 28.0],
                        egui::Button::new(egui::RichText::new(i18n.settings_advanced()).color(
                            if advanced_selected {
                                accent_color
                            } else {
                                ui.style().visuals.text_color()
                            },
                        ))
                        .fill(if advanced_selected {
                            ui.style().visuals.selection.bg_fill
                        } else {
                            crate::ui::theme::transparent_color()
                        }),
                    );
                    if advanced_response.clicked() {
                        state.tab = SettingsTab::Advanced;
                    }

                    ui.add_space(4.0);

                    // 关于
                    let about_selected = state.tab == SettingsTab::About;
                    let about_response = ui.add_sized(
                        [ui.available_width(), 28.0],
                        egui::Button::new(egui::RichText::new(i18n.settings_about()).color(
                            if about_selected {
                                accent_color
                            } else {
                                ui.style().visuals.text_color()
                            },
                        ))
                        .fill(if about_selected {
                            ui.style().visuals.selection.bg_fill
                        } else {
                            crate::ui::theme::transparent_color()
                        }),
                    );
                    if about_response.clicked() {
                        state.tab = SettingsTab::About;
                    }
                });

                ui.separator();

                // 右侧内容
                ui.vertical(|ui| {
                    ui.set_min_width(ui.available_width());
                    egui::ScrollArea::vertical()
                        .id_salt("settings_content")
                        .show(ui, |ui| {
                            ui.add_space(8.0);
                            match state.tab {
                                SettingsTab::Log => {
                                    draw_log_settings(ui, state, i18n);
                                }
                                SettingsTab::Appearance => {
                                    draw_appearance_settings(ctx, ui, &mut state.theme_mode, i18n);
                                }
                                SettingsTab::Advanced => {
                                    draw_advanced_settings(ui, state, i18n);
                                }
                                SettingsTab::Backup => {
                                    draw_backup_settings(ui, state, i18n);
                                }
                                SettingsTab::About => {
                                    download_release = draw_about_content(
                                        ui,
                                        &mut state.about_icon_texture,
                                        update_manager,
                                        i18n,
                                    );
                                }
                            }
                        });
                });
            });
        });

    download_release
}

// 日志设置内容
fn draw_log_settings(ui: &mut egui::Ui, state: &mut SettingsWindowState, i18n: &I18n) {
    let text_subtle = ui.style().visuals.text_color().gamma_multiply(0.6);

    // 日志启用提示
    if crate::logger::is_log_config_changed() {
        let tip_text = if crate::logger::is_log_enabled() {
            i18n.log_enabled_hint()
        } else {
            i18n.log_disabled_hint()
        };
        ui.label(
            egui::RichText::new(tip_text)
                .size(11.0)
                .color(crate::ui::theme::warning_color(ui.ctx())),
        );
        ui.add_space(12.0);
    }

    // 日志启用开关
    let mut log_enabled = crate::logger::is_log_enabled();
    if ui
        .checkbox(&mut log_enabled, i18n.enable_log_storage())
        .changed()
    {
        crate::logger::set_log_enabled(log_enabled);
        if log_enabled {
            tracing::info!("日志存储已启用，将在下次启动时生效");
        } else {
            tracing::info!("日志存储已禁用，将在下次启动时生效");
        }
    }

    // 日志目录路径
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new(i18n.log_dir_label())
            .size(11.0)
            .color(text_subtle),
    );
    ui.add_space(4.0);

    // 获取原始路径用于重置
    let original_path = crate::logger::get_log_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    ui.horizontal(|ui| {
        let response = ui.add_sized(
            [ui.available_width() - 32.0, 20.0],
            egui::TextEdit::singleline(&mut state.log_dir_display),
        );

        // 编辑后自动重置（失去焦点时）
        if response.lost_focus() && state.log_dir_display != original_path {
            state.log_dir_display = original_path.clone();
        }

        if ui
            .button(icons::FOLDER_OPEN)
            .on_hover_text(i18n.open_log_dir())
            .clicked()
            && let Err(e) = crate::logger::open_log_directory()
        {
            tracing::error!("打开日志目录失败: {}", e);
        }
    });
}

// 外观设置内容
fn draw_appearance_settings(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    theme_mode: &mut crate::ui::theme::ThemeMode,
    i18n: &I18n,
) {
    // 主题选择
    ui.horizontal(|ui| {
        ui.label(i18n.theme_mode_label());

        let current_mode = *theme_mode;
        egui::ComboBox::from_id_salt("theme_mode_selector")
            .selected_text(current_mode.display_name(i18n))
            .width(120.0)
            .show_ui(ui, |ui| {
                for mode in crate::ui::theme::ThemeMode::all() {
                    let is_selected = current_mode == *mode;
                    if ui
                        .selectable_label(is_selected, mode.display_name(i18n))
                        .clicked()
                    {
                        *theme_mode = *mode;
                        crate::ui::theme::apply_theme(ctx, *mode);
                    }
                }
            });
    });
}

// 高级设置内容
fn draw_advanced_settings(ui: &mut egui::Ui, state: &mut SettingsWindowState, i18n: &I18n) {
    let text_subtle = ui.style().visuals.text_color().gamma_multiply(0.6);
    let success_color = crate::ui::theme::success_color(ui.ctx());
    let error_color = crate::ui::theme::error_color(ui.ctx());
    let warning_color = crate::ui::theme::warning_color(ui.ctx());

    // Steam 路径设置
    ui.heading(i18n.steam_path_label());
    ui.add_space(8.0);

    // 路径输入框和浏览按钮
    ui.horizontal(|ui| {
        ui.add_sized(
            [ui.available_width() - 60.0, 24.0],
            egui::TextEdit::singleline(&mut state.steam_path_input)
                .hint_text("Steam 安装路径")
                .interactive(false), // 只读，通过浏览按钮修改
        );

        if ui.button(i18n.steam_path_browse()).clicked()
            && let Some(path) = rfd::FileDialog::new().pick_folder()
        {
            state.steam_path_input = path.display().to_string();
            state.steam_path_changed = true;
        }
    });

    // 验证路径并显示状态
    let path = PathBuf::from(&state.steam_path_input);
    let validation = crate::config::validate_steam_path(&path);

    ui.add_space(4.0);
    match &validation {
        crate::config::SteamPathValidation::Valid { user_count } => {
            ui.label(
                egui::RichText::new(i18n.steam_path_valid(*user_count))
                    .size(11.0)
                    .color(success_color),
            );
        }
        crate::config::SteamPathValidation::NotExists => {
            ui.label(
                egui::RichText::new(i18n.steam_path_not_exists())
                    .size(11.0)
                    .color(error_color),
            );
        }
        crate::config::SteamPathValidation::InvalidStructure => {
            ui.label(
                egui::RichText::new(i18n.steam_path_no_userdata())
                    .size(11.0)
                    .color(error_color),
            );
        }
        crate::config::SteamPathValidation::NoUsers => {
            ui.label(
                egui::RichText::new(i18n.steam_path_no_users())
                    .size(11.0)
                    .color(warning_color),
            );
        }
    }

    ui.add_space(8.0);

    // 操作按钮
    ui.horizontal(|ui| {
        // 自动检测按钮
        if ui.button(i18n.steam_path_auto_detect()).clicked() {
            // 清除自定义路径，使用自动检测
            if let Err(e) = crate::config::set_custom_steam_path(None) {
                tracing::error!("清除自定义路径失败: {}", e);
            }
            // 更新显示路径
            if let Ok(detected) = crate::vdf_parser::VdfParser::find_steam_path() {
                state.steam_path_input = detected.display().to_string();
            }
            state.steam_path_changed = true;
        }
    });

    // 显示需要重启提示
    if state.steam_path_changed {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!(
                "{} {}",
                icons::WARNING,
                i18n.steam_path_restart_hint()
            ))
            .size(11.0)
            .color(warning_color),
        );

        // 保存按钮
        if ui.button(i18n.ok()).clicked() && validation.is_valid() {
            if let Err(e) = crate::config::set_custom_steam_path(Some(path.clone())) {
                tracing::error!("保存 Steam 路径失败: {}", e);
            } else {
                tracing::info!("已保存 Steam 路径: {:?}", path);
            }
        }
    }

    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(i18n.steam_path_hint())
            .size(10.0)
            .color(text_subtle),
    );

    ui.add_space(24.0);
    ui.separator();
    ui.add_space(16.0);

    // 恢复默认设置
    ui.horizontal(|ui| {
        if ui
            .button(egui::RichText::new(i18n.reset_all_settings()).color(error_color))
            .clicked()
        {
            state.show_reset_confirm = true;
        }
    });

    // 确认重置对话框
    if state.show_reset_confirm {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(i18n.reset_confirm());
            if ui.button(i18n.ok()).clicked() {
                if let Err(e) = crate::config::reset_to_default() {
                    tracing::error!("重置配置失败: {}", e);
                }
                state.show_reset_confirm = false;
                // 更新状态
                state.steam_path_input.clear();
                if let Ok(path) = crate::vdf_parser::VdfParser::find_steam_path() {
                    state.steam_path_input = path.display().to_string();
                }
                state.steam_path_changed = true;
            }
            if ui.button(i18n.cancel()).clicked() {
                state.show_reset_confirm = false;
            }
        });
    }

    ui.add_space(24.0);

    // 显示配置文件位置
    let original_config_path = crate::config::get_config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let config_path_for_open = crate::config::get_config_path().ok();

    ui.label(
        egui::RichText::new(i18n.config_dir_label())
            .size(11.0)
            .color(text_subtle),
    );
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let response = ui.add_sized(
            [ui.available_width() - 32.0, 20.0],
            egui::TextEdit::singleline(&mut state.config_path_display),
        );

        if response.lost_focus() && state.config_path_display != original_config_path {
            state.config_path_display = original_config_path.clone();
        }

        if ui
            .button(icons::FOLDER_OPEN)
            .on_hover_text(i18n.open_config_dir())
            .clicked()
            && let Some(ref config_path) = config_path_for_open
            && let Some(parent) = config_path.parent()
        {
            let _ = open::that(parent);
        }
    });
}

// 备份设置内容
fn draw_backup_settings(ui: &mut egui::Ui, state: &mut SettingsWindowState, i18n: &I18n) {
    let text_subtle = ui.style().visuals.text_color().gamma_multiply(0.6);

    // 备份目录路径
    let original_backup_path = crate::backup::get_backup_root_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    ui.label(
        egui::RichText::new(i18n.backup_dir_label())
            .size(11.0)
            .color(text_subtle),
    );
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let response = ui.add_sized(
            [ui.available_width() - 32.0, 20.0],
            egui::TextEdit::singleline(&mut state.backup_dir_display),
        );

        if response.lost_focus() && state.backup_dir_display != original_backup_path {
            state.backup_dir_display = original_backup_path.clone();
        }

        if ui
            .button(icons::FOLDER_OPEN)
            .on_hover_text(i18n.backup_open_dir())
            .clicked()
            && let Ok(manager) = crate::backup::BackupManager::new()
            && let Err(e) = manager.open_backup_dir()
        {
            tracing::error!("打开备份目录失败: {}", e);
        }
    });
}

// 关于内容
fn draw_about_content(
    ui: &mut egui::Ui,
    about_icon_texture: &mut Option<egui::TextureHandle>,
    update_manager: &mut crate::update::UpdateManager,
    i18n: &I18n,
) -> Option<crate::update::ReleaseInfo> {
    let steam_blue = crate::ui::theme::accent_color(ui.ctx());
    let text_subtle = ui.style().visuals.text_color().gamma_multiply(0.6);
    let text_normal = ui.style().visuals.text_color();

    let mut download_release = None;

    ui.vertical_centered(|ui| {
        // 加载应用图标
        if about_icon_texture.is_none() {
            let icon_bytes =
                include_bytes!("../../assets/steam_cloud-macOS-Default-1024x1024@1x.png");
            if let Ok(img) = image::load_from_memory(icon_bytes) {
                let img = img.resize_exact(96, 96, image::imageops::FilterType::Lanczos3);
                let rgba = img.to_rgba8();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [96, 96],
                    rgba.as_flat_samples().as_slice(),
                );
                *about_icon_texture = Some(ui.ctx().load_texture(
                    "settings_about_icon",
                    color_image,
                    Default::default(),
                ));
            }
        }

        if let Some(texture) = about_icon_texture.as_ref() {
            ui.image(texture);
        }

        ui.add_space(12.0);

        ui.label(
            egui::RichText::new("Steam Cloud File Manager")
                .size(18.0)
                .strong()
                .color(text_normal),
        );
    });

    ui.add_space(16.0);

    // 版本信息
    egui::Grid::new("about_info_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            let row = |ui: &mut egui::Ui, key: &str, val: String| {
                ui.label(egui::RichText::new(key).size(12.0).color(text_subtle));
                ui.label(
                    egui::RichText::new(val)
                        .size(12.0)
                        .color(text_normal)
                        .monospace(),
                );
                ui.end_row();
            };

            row(ui, "Version", crate::version::full_version().to_string());
            row(
                ui,
                "OS",
                format!(
                    "{} ({})",
                    crate::version::os_name(),
                    crate::version::os_arch()
                ),
            );
            row(
                ui,
                "Build",
                format!(
                    "{} - {}",
                    crate::version::build_profile(),
                    crate::version::build_time()
                ),
            );
        });

    ui.add_space(12.0);

    // 检查更新按钮
    ui.horizontal(|ui| {
        let checking = matches!(
            update_manager.status(),
            crate::update::UpdateStatus::Checking
        );
        let button_text = if checking {
            i18n.checking_update()
        } else {
            i18n.check_update_btn()
        };

        if ui
            .add_enabled(!checking, egui::Button::new(button_text))
            .clicked()
        {
            let _ = update_manager.check_update();
        }

        // 更新状态显示
        let update_status = update_manager.status().clone();
        match &update_status {
            crate::update::UpdateStatus::NoUpdate => {
                ui.label(
                    egui::RichText::new(i18n.already_latest())
                        .size(11.0)
                        .color(crate::ui::theme::success_color(ui.ctx())),
                );
            }
            crate::update::UpdateStatus::Available(release) => {
                ui.label(
                    egui::RichText::new(i18n.new_version_found(&release.tag_name))
                        .size(11.0)
                        .color(crate::ui::theme::warning_color(ui.ctx())),
                );
            }
            crate::update::UpdateStatus::Error(err) => {
                ui.label(
                    egui::RichText::new(format!("{} {}", icons::ERROR, err))
                        .size(10.0)
                        .color(crate::ui::theme::error_color(ui.ctx())),
                );
            }
            _ => {}
        }
    });

    // 更新操作区域
    let update_status = update_manager.status().clone();
    if let crate::update::UpdateStatus::Available(release) = &update_status {
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        let mut should_open_page = false;

        ui.label(
            egui::RichText::new(i18n.new_version_hint())
                .size(11.0)
                .color(text_subtle),
        );

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let button_text = i18n.download_and_install();

            if ui.button(button_text).clicked() {
                download_release = Some(release.clone());
            }
            if ui.button(i18n.view_details()).clicked() {
                should_open_page = true;
            }
        });

        if should_open_page {
            crate::update::UpdateManager::open_release_page();
        }
    }

    // 下载/安装进度
    match &update_status {
        crate::update::UpdateStatus::Downloading(progress) => {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(i18n.downloading_update())
                    .size(12.0)
                    .color(steam_blue),
            );
            ui.add(egui::ProgressBar::new(*progress).show_percentage());
        }
        crate::update::UpdateStatus::Installing => {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(i18n.installing_update())
                    .size(12.0)
                    .color(steam_blue),
            );
        }
        crate::update::UpdateStatus::Success => {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(i18n.update_success())
                    .size(12.0)
                    .color(crate::ui::theme::success_color(ui.ctx())),
            );
            ui.label(
                egui::RichText::new(i18n.restart_to_apply())
                    .size(10.0)
                    .color(text_subtle),
            );
            if ui.button(i18n.restart_now()).clicked() {
                std::process::exit(0);
            }
        }
        _ => {}
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(12.0);

    // 链接
    egui::Grid::new("about_links_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(i18n.author())
                    .size(11.0)
                    .color(text_subtle),
            );
            ui.hyperlink_to(
                egui::RichText::new("Flacier").size(11.0).color(steam_blue),
                "https://github.com/Fldicoahkiin",
            );
            ui.end_row();

            ui.label(
                egui::RichText::new(i18n.github_repository())
                    .size(11.0)
                    .color(text_subtle),
            );
            ui.hyperlink_to(
                egui::RichText::new("Fldicoahkiin/SteamCloudFileManager")
                    .size(11.0)
                    .color(steam_blue),
                "https://github.com/Fldicoahkiin/SteamCloudFileManager",
            );
            ui.end_row();
        });

    ui.add_space(12.0);

    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Copyright © 2026 Flacier")
                .size(9.0)
                .color(text_subtle),
        );
        ui.label(
            egui::RichText::new("GPL-3.0 License · Powered by Rust & egui")
                .size(9.0)
                .color(text_subtle),
        );
    });

    download_release
}
