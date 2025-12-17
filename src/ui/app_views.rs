use crate::i18n::I18n;
use eframe::egui;

// 绘制未连接状态的提示
pub fn draw_disconnected_view(ui: &mut egui::Ui, i18n: &I18n) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 2.0 - 80.0);
        ui.heading(i18n.status_enter_app_id());
        ui.add_space(20.0);
        ui.label(i18n.hint_you_can());
        ui.label(i18n.hint_select_game());
        ui.label(i18n.hint_enter_app_id());
    });
}

// 绘制连接中/加载中状态
pub fn draw_loading_view(ui: &mut egui::Ui, is_connecting: bool, i18n: &I18n) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 2.0 - 40.0);
        ui.spinner();
        ui.add_space(10.0);
        if is_connecting {
            ui.label(i18n.connecting());
        } else {
            ui.label(i18n.status_loading_files());
        }
    });
}

// 绘制无文件状态
pub fn draw_no_files_view(ui: &mut egui::Ui, i18n: &I18n) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 2.0 - 50.0);
        ui.heading(i18n.no_cloud_files());
        ui.add_space(10.0);
        ui.label(i18n.no_cloud_files_hint());
    });
}
