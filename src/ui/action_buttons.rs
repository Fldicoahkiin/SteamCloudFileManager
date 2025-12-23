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
    SyncToCloud,
    CompareFiles,
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
        if ui
            .button(i18n.select_all())
            .on_hover_text(i18n.select_all_hint())
            .clicked()
        {
            action = FileAction::SelectAll;
        }

        if ui
            .button(i18n.invert_selection())
            .on_hover_text(i18n.invert_selection_hint())
            .clicked()
        {
            action = FileAction::InvertSelection;
        }

        if ui
            .button(i18n.clear_selection())
            .on_hover_text(i18n.clear_selection_hint())
            .clicked()
        {
            action = FileAction::ClearSelection;
        }

        ui.separator();

        // 文件操作
        if ui
            .add_enabled(
                can_operate && has_selection,
                egui::Button::new(i18n.download()),
            )
            .on_hover_text(i18n.download_hint())
            .clicked()
        {
            action = FileAction::DownloadSelected;
        }

        if ui
            .add_enabled(can_operate, egui::Button::new(i18n.upload()))
            .on_hover_text(i18n.upload_hint())
            .clicked()
        {
            action = FileAction::Upload;
        }

        if ui
            .add_enabled(
                can_operate && has_selection,
                egui::Button::new(i18n.sync_to_cloud()),
            )
            .on_hover_text(i18n.sync_to_cloud_hint())
            .clicked()
        {
            action = FileAction::SyncToCloud;
        }

        if ui
            .add_enabled(
                can_operate && has_selection,
                egui::Button::new(i18n.delete()),
            )
            .on_hover_text(i18n.delete_hint())
            .clicked()
        {
            action = FileAction::DeleteSelected;
        }

        if ui
            .add_enabled(
                can_operate && has_selection,
                egui::Button::new(i18n.forget()),
            )
            .on_hover_text(i18n.forget_hint())
            .clicked()
        {
            action = FileAction::ForgetSelected;
        }

        ui.separator();

        // 文件对比
        if ui
            .add_enabled(can_operate, egui::Button::new(i18n.compare_files()))
            .on_hover_text(i18n.compare_files_hint())
            .clicked()
        {
            action = FileAction::CompareFiles;
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
