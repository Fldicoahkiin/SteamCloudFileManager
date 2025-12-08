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
    total_count: usize,
    selected_total_size: u64,
) -> FileAction {
    let mut action = FileAction::None;

    ui.horizontal(|ui| {
        // 选择操作
        if ui.button("全选").clicked() {
            action = FileAction::SelectAll;
        }

        if ui.button("反选").clicked() {
            action = FileAction::InvertSelection;
        }

        if ui.button("清除选择").clicked() {
            action = FileAction::ClearSelection;
        }

        ui.separator();

        // 文件操作
        if ui
            .add_enabled(can_operate && has_selection, egui::Button::new("下载"))
            .clicked()
        {
            action = FileAction::DownloadSelected;
        }

        if ui
            .add_enabled(can_operate, egui::Button::new("上传"))
            .on_hover_text("选择文件或文件夹进行上传")
            .clicked()
        {
            action = FileAction::Upload;
        }

        if ui
            .add_enabled(can_operate && has_selection, egui::Button::new("删除"))
            .clicked()
        {
            action = FileAction::DeleteSelected;
        }

        if ui
            .add_enabled(can_operate, egui::Button::new("取消云同步"))
            .clicked()
        {
            action = FileAction::ForgetSelected;
        }

        // 右侧统计信息
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("已选: {}/{}", selected_count, total_count));

            if selected_count > 0 {
                ui.label(format!(
                    "总大小: {}",
                    crate::utils::format_size(selected_total_size)
                ));
            }
        });
    });

    action
}
