use crate::i18n::I18n;
use egui;

// 绘制调试警告横幅
pub fn draw_debug_warning_ui(ui: &mut egui::Ui, i18n: &I18n) -> (bool, bool, bool) {
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
                egui::RichText::new(i18n.debug_mode_not_enabled())
                    .color(egui::Color32::from_rgb(255, 200, 0))
                    .size(16.0)
                    .strong(),
            );
        });

        // Steam 运行状态
        ui.horizontal(|ui| {
            if steam_running {
                ui.label(
                    egui::RichText::new(i18n.steam_running())
                        .color(egui::Color32::from_rgb(100, 200, 100))
                        .size(13.0),
                );
            } else {
                ui.label(
                    egui::RichText::new(i18n.steam_not_running())
                        .color(egui::Color32::from_rgb(200, 100, 100))
                        .size(13.0),
                );
            }
        });

        // 说明文字
        ui.label(
            egui::RichText::new(i18n.debug_mode_hint())
                .color(egui::Color32::LIGHT_GRAY)
                .size(13.0),
        );

        ui.add_space(4.0);

        // 操作按钮组
        ui.horizontal(|ui| {
            // 自动重启按钮
            let button_text = if steam_running {
                i18n.auto_restart_steam()
            } else {
                i18n.start_steam()
            };
            let hover_text = if steam_running {
                i18n.auto_restart_hint()
            } else {
                i18n.start_steam_hint()
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
                .button(egui::RichText::new(i18n.view_manual_steps()).size(14.0))
                .on_hover_text(i18n.manual_steps_hint())
                .clicked()
            {
                show_manual = true;
            }

            ui.separator();

            // 暂时忽略按钮
            if ui
                .button(
                    egui::RichText::new(i18n.dismiss_temporarily())
                        .size(14.0)
                        .color(egui::Color32::GRAY),
                )
                .on_hover_text(i18n.dismiss_hint())
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

// 工具栏按钮状态
pub struct ToolbarState<'a> {
    pub user_id: Option<&'a str>,
    pub has_files: bool,
    pub on_settings: &'a mut bool,
    pub on_user_selector: &'a mut bool,
    pub on_game_selector: &'a mut bool,
    pub on_backup: &'a mut bool,
}

// 绘制顶部工具栏按钮组
pub fn draw_toolbar_buttons(ui: &mut egui::Ui, state: &mut ToolbarState, i18n: &I18n) {
    if ui
        .button(i18n.settings_title())
        .on_hover_text(i18n.settings_title())
        .clicked()
    {
        *state.on_settings = true;
    }

    ui.separator();

    if ui
        .button(i18n.select_account())
        .on_hover_text(i18n.select_account_hint())
        .clicked()
    {
        *state.on_user_selector = true;
    }

    if ui
        .button(i18n.select_game())
        .on_hover_text(i18n.select_game_hint())
        .clicked()
    {
        *state.on_game_selector = true;
    }

    ui.separator();

    // 备份按钮
    if ui
        .add_enabled(state.has_files, egui::Button::new(i18n.backup()))
        .on_hover_text(i18n.backup_hint())
        .clicked()
    {
        *state.on_backup = true;
    }

    ui.separator();

    if let Some(user_id) = state.user_id {
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
        if ui
            .button(i18n.disconnect())
            .on_hover_text(i18n.disconnect_hint())
            .clicked()
        {
            action = ConnectionAction::Disconnect;
        }

        // 刷新按钮：打开对应 appid 的云存储页面
        if ui
            .button(i18n.refresh())
            .on_hover_text(i18n.refresh_open_url_hint())
            .clicked()
        {
            action = ConnectionAction::Refresh;
        }

        // 连接时提示：断开后 Steam 将自动同步
        ui.label(
            egui::RichText::new(i18n.disconnect_sync_hint())
                .color(egui::Color32::from_rgb(102, 192, 244)),
        );
    } else if ui
        .button(i18n.connect())
        .on_hover_text(i18n.connect_hint())
        .clicked()
    {
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

// 文件选择辅助函数
pub fn select_all_files(file_count: usize) -> Vec<usize> {
    (0..file_count).collect()
}

pub fn invert_file_selection(current_selected: &[usize], file_count: usize) -> Vec<usize> {
    let current_set: std::collections::HashSet<_> = current_selected.iter().copied().collect();
    (0..file_count)
        .filter(|i| !current_set.contains(i))
        .collect()
}

pub fn clear_file_selection() -> Vec<usize> {
    Vec::new()
}
