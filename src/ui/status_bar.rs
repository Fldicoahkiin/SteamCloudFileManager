use crate::i18n::I18n;
use egui;

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
        let text = i18n.quota_usage(usage_percent, &used_str, &total_str);
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
        ui.label(i18n.status_label());
        ui.label(status_message);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(enabled) = cloud_enabled {
                let cloud_status = if enabled {
                    i18n.cloud_on()
                } else {
                    i18n.cloud_off()
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
#[derive(Debug, Clone, PartialEq)]
pub enum StatusPanelAction {
    None,
    ToggleCloudEnabled,
    ShowAppInfo(u32),
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
    pub app_id: u32,
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

    // 显示 appinfo.vdf 按钮
    if state.is_connected && state.app_id > 0 {
        ui.horizontal(|ui| {
            if ui.button(i18n.show_appinfo_vdf()).clicked() {
                action = StatusPanelAction::ShowAppInfo(state.app_id);
            }
        });
    }

    action
}
