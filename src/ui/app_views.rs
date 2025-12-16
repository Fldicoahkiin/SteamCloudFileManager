use eframe::egui;

// 绘制未连接状态的提示
pub fn draw_disconnected_view(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 2.0 - 80.0);
        ui.heading("请输入 App ID 并连接到 Steam");
        ui.add_space(20.0);
        ui.label("您可以：");
        ui.label("点击上方的 '游戏库' 按钮选择游戏");
        ui.label("或直接输入 App ID 并点击 '连接'");
    });
}

// 绘制连接中/加载中状态
pub fn draw_loading_view(ui: &mut egui::Ui, is_connecting: bool) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 2.0 - 40.0);
        ui.spinner();
        ui.add_space(10.0);
        if is_connecting {
            ui.label("正在连接到 Steam...");
        } else {
            ui.label("正在加载文件列表...");
        }
    });
}

// 绘制无文件状态
pub fn draw_no_files_view(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 2.0 - 50.0);
        ui.heading("没有找到云文件");
        ui.add_space(10.0);
        ui.label("该游戏没有云存档文件");
    });
}
