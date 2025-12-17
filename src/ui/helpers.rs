use crate::i18n::I18n;
use egui;

// 绘制调试警告横幅
pub fn draw_debug_warning_ui(ui: &mut egui::Ui) -> (bool, bool, bool) {
    let mut restart_clicked = false;
    let mut dismiss_clicked = false;
    let mut show_manual = false;

    // 检测 Steam 运行状态
    let steam_running = crate::steam_process::is_steam_running();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 8.0;

        // 警告标题
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("⚠ Steam 调试模式未启用")
                    .color(egui::Color32::from_rgb(255, 200, 0))
                    .size(16.0)
                    .strong(),
            );
        });

        // Steam 运行状态
        ui.horizontal(|ui| {
            if steam_running {
                ui.label(
                    egui::RichText::new("✓ Steam 正在运行")
                        .color(egui::Color32::from_rgb(100, 200, 100))
                        .size(13.0),
                );
            } else {
                ui.label(
                    egui::RichText::new("✗ Steam 未运行")
                        .color(egui::Color32::from_rgb(200, 100, 100))
                        .size(13.0),
                );
            }
        });

        // 说明文字
        ui.label(
            egui::RichText::new("需要启用 Steam 的 CEF 调试模式才能使用网页登录功能")
                .color(egui::Color32::LIGHT_GRAY)
                .size(13.0),
        );

        ui.add_space(4.0);

        // 操作按钮组
        ui.horizontal(|ui| {
            // 自动重启按钮
            let button_text = if steam_running {
                "自动重启 Steam"
            } else {
                "启动 Steam"
            };
            let hover_text = if steam_running {
                "自动关闭并重启 Steam，添加调试参数"
            } else {
                "以调试模式启动 Steam"
            };

            if ui
                .button(egui::RichText::new(button_text).size(14.0))
                .on_hover_text(hover_text)
                .clicked()
            {
                restart_clicked = true;
            }

            ui.separator();

            // 手动操作按钮
            if ui
                .button(egui::RichText::new("查看手动操作").size(14.0))
                .on_hover_text("显示如何手动添加启动参数")
                .clicked()
            {
                show_manual = true;
            }

            ui.separator();

            // 暂时忽略按钮
            if ui
                .button(
                    egui::RichText::new("✕ 暂时忽略")
                        .size(14.0)
                        .color(egui::Color32::GRAY),
                )
                .on_hover_text("隐藏此提示（可在设置中重新显示）")
                .clicked()
            {
                dismiss_clicked = true;
            }
        });
    });

    (restart_clicked, dismiss_clicked, show_manual)
}

// 获取手动操作指南对话框
pub fn get_manual_guide_dialog(i18n: &crate::i18n::I18n) -> crate::ui::GuideDialog {
    #[cfg(target_os = "macos")]
    {
        crate::ui::create_macos_manual_guide(i18n)
    }

    #[cfg(target_os = "windows")]
    {
        crate::ui::create_windows_manual_guide(i18n)
    }

    #[cfg(target_os = "linux")]
    {
        crate::ui::create_linux_manual_guide(i18n)
    }
}

// 绘制顶部工具栏按钮组
pub fn draw_toolbar_buttons(
    ui: &mut egui::Ui,
    user_id: Option<&str>,
    on_about: &mut bool,
    on_user_selector: &mut bool,
    on_game_selector: &mut bool,
    i18n: &I18n,
) {
    if ui.button(i18n.about_title()).clicked() {
        *on_about = true;
    }

    ui.separator();

    if ui.button(i18n.select_account()).clicked() {
        *on_user_selector = true;
    }

    if ui.button(i18n.select_game()).clicked() {
        *on_game_selector = true;
    }

    ui.separator();

    if let Some(user_id) = user_id {
        ui.label(format!("{}: {}", i18n.select_account(), user_id));
        ui.separator();
    }
}

// 绘制 App ID 输入和连接按钮
pub fn draw_connection_controls(
    ui: &mut egui::Ui,
    app_id_input: &mut String,
    is_connected: bool,
    is_connecting: bool,
    i18n: &I18n,
) -> ConnectionAction {
    ui.label("App ID:");
    let response = ui.add(egui::TextEdit::singleline(app_id_input).desired_width(150.0));

    let mut action = ConnectionAction::None;

    if response.changed() {
        action = ConnectionAction::InputChanged;
    }

    if is_connected {
        if ui.button(i18n.disconnect()).clicked() {
            action = ConnectionAction::Disconnect;
        }

        if ui.button(i18n.refresh()).clicked() {
            action = ConnectionAction::Refresh;
        }
    } else if ui.button(i18n.connect()).clicked() {
        action = ConnectionAction::Connect;
    }

    if is_connecting {
        ui.spinner();
    }

    action
}

#[derive(Debug, PartialEq)]
pub enum ConnectionAction {
    None,
    InputChanged,
    Connect,
    Disconnect,
    Refresh,
}

// 绘制云存储状态信息
pub fn draw_cloud_status(
    ui: &mut egui::Ui,
    account_enabled: Option<bool>,
    _app_enabled: Option<bool>,
    i18n: &I18n,
) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", i18n.account_cloud_status()));
        match account_enabled {
            Some(true) => ui.label(format!("✅ {}", i18n.logged_in())),
            Some(false) => ui.label(format!("❌ {}", i18n.not_logged_in())),
            None => ui.label("❓ Unknown"),
        };
    });
}

// 绘制配额信息
pub fn draw_quota_info(ui: &mut egui::Ui, total: u64, available: u64, i18n: &I18n) {
    ui.horizontal(|ui| {
        let used = total - available;
        let usage_percent = (used as f32 / total as f32 * 100.0).round();
        let used_str = crate::file_manager::format_size(used);
        let total_str = crate::file_manager::format_size(total);
        let text = match i18n.language() {
            crate::i18n::Language::Chinese => format!(
                "配额: {:.1}% 已使用 ({}/{})",
                usage_percent, used_str, total_str
            ),
            crate::i18n::Language::English => format!(
                "Quota: {:.1}% used ({}/{})",
                usage_percent, used_str, total_str
            ),
        };
        ui.label(text);
    });
}

// 绘制状态消息栏
pub fn draw_status_message(
    ui: &mut egui::Ui,
    status_message: &str,
    cloud_enabled: Option<bool>,
    i18n: &I18n,
) -> bool {
    let mut toggled = false;
    ui.horizontal(|ui| {
        let status_label = match i18n.language() {
            crate::i18n::Language::Chinese => "状态:",
            crate::i18n::Language::English => "Status:",
        };
        ui.label(status_label);
        ui.label(status_message);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(enabled) = cloud_enabled {
                let cloud_status = match i18n.language() {
                    crate::i18n::Language::Chinese => {
                        if enabled {
                            "云存储: 开启"
                        } else {
                            "云存储: 关闭"
                        }
                    }
                    crate::i18n::Language::English => {
                        if enabled {
                            "Cloud: On"
                        } else {
                            "Cloud: Off"
                        }
                    }
                };
                if ui.selectable_label(false, cloud_status).clicked() {
                    toggled = true;
                }
            }
        });
    });
    toggled
}

// 状态面板的用户操作
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusPanelAction {
    None,
    ToggleCloudEnabled,
}

// 状态面板的状态数据
pub struct StatusPanelState {
    pub status_message: String,
    pub cloud_enabled: Option<bool>,
    pub is_connected: bool,
    pub remote_ready: bool,
    pub account_enabled: Option<bool>,
    pub app_enabled: Option<bool>,
    pub quota_info: Option<(u64, u64)>,
}

// 绘制完整的状态面板
pub fn draw_complete_status_panel(
    ui: &mut egui::Ui,
    state: &StatusPanelState,
    i18n: &I18n,
) -> StatusPanelAction {
    let mut action = StatusPanelAction::None;

    ui.separator();

    // 状态消息栏
    let toggled = draw_status_message(ui, &state.status_message, state.cloud_enabled, i18n);
    if toggled {
        action = StatusPanelAction::ToggleCloudEnabled;
    }

    // 云存储状态
    if state.is_connected {
        if state.remote_ready {
            draw_cloud_status(ui, state.account_enabled, state.app_enabled, i18n);
        } else {
            ui.horizontal(|ui| {
                let text = match i18n.language() {
                    crate::i18n::Language::Chinese => "云存储状态: 未就绪",
                    crate::i18n::Language::English => "Cloud Status: Not Ready",
                };
                ui.label(text);
            });
        }
    }

    // 配额信息
    if let Some((total, available)) = state.quota_info {
        draw_quota_info(ui, total, available, i18n);
    }

    action
}
