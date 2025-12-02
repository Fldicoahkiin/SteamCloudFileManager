use egui;

// 绘制调试警告横幅
pub fn draw_debug_warning_ui(ui: &mut egui::Ui) -> (bool, bool) {
    let mut restart_clicked = false;
    let mut dismiss_clicked = false;

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
                show_manual_guide();
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

    (restart_clicked, dismiss_clicked)
}

// 显示手动操作指南
fn show_manual_guide() {
    #[cfg(target_os = "macos")]
    {
        let guide = "macOS 手动操作步骤：\n\n\
            1. 完全退出 Steam（右键 Dock 图标 -> 退出）\n\
            2. 打开终端（Terminal）\n\
            3. 输入命令：open -a Steam --args -cef-enable-debugging\n\
            4. 等待 Steam 启动完成\n\
            5. 返回本应用重新连接";

        tracing::info!("显示手动操作指南：\n{}", guide);
    }

    #[cfg(target_os = "windows")]
    {
        let guide = "Windows 手动操作步骤：\n\n\
            1. 完全退出 Steam\n\
            2. 右键 Steam 快捷方式 -> 属性\n\
            3. 在目标栏末尾添加：-cef-enable-debugging\n\
            4. 点击确定并启动 Steam\n\
            5. 返回本应用重新连接";

        tracing::info!("显示手动操作指南：\n{}", guide);
    }

    #[cfg(target_os = "linux")]
    {
        let guide = "Linux 手动操作步骤：\n\n\
            1. 完全退出 Steam\n\
            2. 打开终端\n\
            3. 输入命令：steam -cef-enable-debugging\n\
            4. 等待 Steam 启动完成\n\
            5. 返回本应用重新连接";

        tracing::info!("显示手动操作指南：\n{}", guide);
    }
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
