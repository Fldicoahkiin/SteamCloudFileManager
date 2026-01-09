use crate::i18n::I18n;
use crate::vdf_parser::UfsConfig;

// AppInfo 对话框状态
#[derive(Clone)]
pub struct AppInfoDialog {
    pub app_id: u32,
    pub config: UfsConfig,
}

impl AppInfoDialog {
    pub fn new(app_id: u32, config: UfsConfig) -> Self {
        Self { app_id, config }
    }
}

// 绘制 AppInfo 对话框
pub fn draw_appinfo_dialog(ctx: &egui::Context, dialog: &AppInfoDialog, i18n: &I18n) -> bool {
    let mut open = true;

    let title = match i18n.language() {
        crate::i18n::Language::Chinese => format!("appinfo.vdf - App {}", dialog.app_id),
        crate::i18n::Language::English => format!("appinfo.vdf - App {}", dialog.app_id),
    };

    egui::Window::new(title)
        .open(&mut open)
        .resizable(true)
        .default_width(500.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            // 配额信息
            ui.horizontal(|ui| {
                let quota_label = match i18n.language() {
                    crate::i18n::Language::Chinese => "配额:",
                    crate::i18n::Language::English => "Quota:",
                };
                ui.label(quota_label);
                ui.label(crate::file_manager::format_size(dialog.config.quota));

                ui.separator();

                let maxfiles_label = match i18n.language() {
                    crate::i18n::Language::Chinese => "最大文件数:",
                    crate::i18n::Language::English => "Max Files:",
                };
                ui.label(maxfiles_label);
                ui.label(format!("{}", dialog.config.maxnumfiles));
            });

            ui.separator();

            // UFS 配置文本
            let ufs_label = match i18n.language() {
                crate::i18n::Language::Chinese => "UFS 云存储配置:",
                crate::i18n::Language::English => "UFS Cloud Config:",
            };
            ui.label(ufs_label);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut dialog.config.raw_text.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY),
                    );
                });
        });

    open
}
