use crate::i18n::I18n;
use egui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileAction {
    None,
    SelectAll,
    InvertSelection,
    ClearSelection,
    DownloadSelected,
    Upload,
    DeleteSelected,
    ForgetSelected,
}

// 绘制文件操作按钮栏
pub fn draw_file_action_buttons(
    ui: &mut egui::Ui,
    can_operate: bool,
    has_selection: bool,
    selected_count: usize,
    _total_count: usize,
    selected_total_size: u64,
    i18n: &I18n,
) -> FileAction {
    let mut action = FileAction::None;

    ui.horizontal(|ui| {
        // 选择操作
        if ui.button(i18n.select_all()).clicked() {
            action = FileAction::SelectAll;
        }

        if ui.button(i18n.invert_selection()).clicked() {
            action = FileAction::InvertSelection;
        }

        if ui.button(i18n.clear_selection()).clicked() {
            action = FileAction::ClearSelection;
        }

        ui.separator();

        // 文件操作
        if ui
            .add_enabled(
                can_operate && has_selection,
                egui::Button::new(i18n.download()),
            )
            .clicked()
        {
            action = FileAction::DownloadSelected;
        }

        if ui
            .add_enabled(can_operate, egui::Button::new(i18n.upload()))
            .on_hover_text(i18n.upload_tooltip())
            .clicked()
        {
            action = FileAction::Upload;
        }

        if ui
            .add_enabled(
                can_operate && has_selection,
                egui::Button::new(i18n.delete()),
            )
            .clicked()
        {
            action = FileAction::DeleteSelected;
        }

        if ui
            .add_enabled(can_operate, egui::Button::new(i18n.forget()))
            .clicked()
        {
            action = FileAction::ForgetSelected;
        }

        // 右侧统计信息
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(i18n.selected_count(selected_count));

            if selected_count > 0 {
                let size_str = crate::file_manager::format_size(selected_total_size);
                ui.label(i18n.total_size_label(&size_str));
            }
        });
    });

    action
}
