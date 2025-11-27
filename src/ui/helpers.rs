use egui;

// 绘制调试警告横幅
pub fn draw_debug_warning(ui: &mut egui::Ui, on_restart: impl FnOnce()) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("⚠ 注意：未检测到 Steam 调试模式").color(egui::Color32::YELLOW),
        );
        if ui.button("重启 Steam (开启调试)").clicked() {
            clicked = true;
        }
    });
    if clicked {
        on_restart();
    }
    clicked
}

// 绘制顶部工具栏按钮组
pub fn draw_toolbar_buttons(
    ui: &mut egui::Ui,
    user_id: Option<&str>,
    on_about: &mut bool,
    on_user_selector: &mut bool,
    on_game_selector: &mut bool,
) {
    if ui.button("关于").clicked() {
        *on_about = true;
    }

    ui.separator();

    if ui.button("用户").clicked() {
        *on_user_selector = true;
    }

    if ui.button("游戏库").clicked() {
        *on_game_selector = true;
    }

    ui.separator();

    if let Some(user_id) = user_id {
        ui.label(format!("用户: {}", user_id));
        ui.separator();
    }
}

// 绘制 App ID 输入和连接按钮
pub fn draw_connection_controls(
    ui: &mut egui::Ui,
    app_id_input: &mut String,
    is_connected: bool,
    is_connecting: bool,
) -> ConnectionAction {
    ui.label("App ID:");
    let response = ui.add(egui::TextEdit::singleline(app_id_input).desired_width(150.0));

    let mut action = ConnectionAction::None;

    if response.changed() {
        action = ConnectionAction::InputChanged;
    }

    if is_connected {
        if ui.button("断开").clicked() {
            action = ConnectionAction::Disconnect;
        }
    } else if ui.button("连接").clicked() {
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
}

// 绘制云存储状态信息
pub fn draw_cloud_status(
    ui: &mut egui::Ui,
    account_enabled: Option<bool>,
    app_enabled: Option<bool>,
) {
    ui.horizontal(|ui| {
        ui.label("账户云存储:");
        match account_enabled {
            Some(true) => ui.label("✅ 已启用"),
            Some(false) => ui.label("❌ 已禁用"),
            None => ui.label("❓ 未知"),
        };
    });

    ui.horizontal(|ui| {
        ui.label("应用云存储:");
        match app_enabled {
            Some(true) => ui.label("✅ 已启用"),
            Some(false) => ui.label("❌ 已禁用"),
            None => ui.label("❓ 未知"),
        };
    });
}

// 绘制配额信息
pub fn draw_quota_info(ui: &mut egui::Ui, total: u64, available: u64) {
    ui.horizontal(|ui| {
        ui.label("配额:");
        let used = total - available;
        let usage_percent = (used as f32 / total as f32 * 100.0).round();
        let used_str = crate::utils::format_size(used);
        let total_str = crate::utils::format_size(total);
        ui.label(format!(
            "{:.1}% 已使用 ({}/{})",
            usage_percent, used_str, total_str
        ));
    });
}

// 绘制状态消息栏
pub fn draw_status_message(
    ui: &mut egui::Ui,
    status_message: &str,
    cloud_enabled: Option<bool>,
) -> bool {
    let mut toggled = false;
    ui.horizontal(|ui| {
        ui.label("状态:");
        ui.label(status_message);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(enabled) = cloud_enabled {
                let cloud_status = if enabled {
                    "云存储: 开启"
                } else {
                    "云存储: 关闭"
                };
                if ui.selectable_label(false, cloud_status).clicked() {
                    toggled = true;
                }
            }
        });
    });
    toggled
}
