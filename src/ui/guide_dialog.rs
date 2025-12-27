use crate::i18n::I18n;
use egui;

#[derive(Debug, Clone, PartialEq)]
pub enum GuideDialogType {
    RestartProgress {
        status: String,
        is_success: bool,
        is_error: bool,
    },
    ManualOperation {
        title: String,
        steps: Vec<String>,
    },
}

pub struct GuideDialog {
    pub dialog_type: GuideDialogType,
    pub show: bool,
}

impl GuideDialog {
    pub fn new(dialog_type: GuideDialogType) -> Self {
        Self {
            dialog_type,
            show: true,
        }
    }

    pub fn update_status(&mut self, status: String, is_success: bool, is_error: bool) {
        if let GuideDialogType::RestartProgress { .. } = &self.dialog_type {
            self.dialog_type = GuideDialogType::RestartProgress {
                status,
                is_success,
                is_error,
            };
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context, i18n: &I18n) -> GuideDialogAction {
        let mut action = GuideDialogAction::None;

        let title = match &self.dialog_type {
            GuideDialogType::RestartProgress { .. } => i18n.restarting_steam().to_string(),
            GuideDialogType::ManualOperation { title, .. } => title.clone(),
        };

        egui::Window::new(title)
            .open(&mut self.show)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| match &self.dialog_type {
                GuideDialogType::RestartProgress {
                    status,
                    is_success,
                    is_error,
                } => {
                    ui.add_space(10.0);

                    if *is_error {
                        ui.colored_label(crate::ui::theme::error_color(ctx), status);
                    } else if *is_success {
                        ui.heading(status);
                    } else {
                        ui.label(status);
                        ui.add_space(5.0);
                        ui.spinner();
                    }

                    ui.add_space(15.0);

                    if *is_success || *is_error {
                        ui.separator();
                        ui.add_space(10.0);
                        if ui.button(i18n.ok()).clicked() {
                            action = GuideDialogAction::Confirm;
                        }
                    }
                }
                GuideDialogType::ManualOperation { steps, .. } => {
                    ui.label(i18n.manual_operation_required());
                    ui.add_space(10.0);

                    for (i, step) in steps.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}.", i + 1));
                            ui.label(step);
                        });
                    }

                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);

                    if ui.button(i18n.i_understand()).clicked() {
                        action = GuideDialogAction::Confirm;
                    }
                }
            });

        action
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuideDialogAction {
    None,
    Confirm,
}

pub fn create_restart_progress_dialog(status: String) -> GuideDialog {
    GuideDialog::new(GuideDialogType::RestartProgress {
        status,
        is_success: false,
        is_error: false,
    })
}

#[cfg(target_os = "macos")]
pub fn create_macos_manual_guide(i18n: &I18n) -> GuideDialog {
    let steps = match i18n.language() {
        crate::i18n::Language::Chinese => vec![
            "右键点击 Dock 中的 Steam 图标，选择「退出」".to_string(),
            "打开「终端」应用（可在启动台中搜索）".to_string(),
            "在终端中输入并执行：".to_string(),
            "  open -a Steam --args -cef-enable-debugging".to_string(),
            "等待 Steam 启动完成".to_string(),
        ],
        crate::i18n::Language::English => vec![
            "Right-click Steam icon in Dock, select 'Quit'".to_string(),
            "Open 'Terminal' app (search in Launchpad)".to_string(),
            "Enter and execute in terminal:".to_string(),
            "  open -a Steam --args -cef-enable-debugging".to_string(),
            "Wait for Steam to start".to_string(),
        ],
    };
    GuideDialog::new(GuideDialogType::ManualOperation {
        title: i18n.manual_restart_macos_title().to_string(),
        steps,
    })
}

#[cfg(target_os = "windows")]
pub fn create_windows_manual_guide(i18n: &I18n) -> GuideDialog {
    let steps = match i18n.language() {
        crate::i18n::Language::Chinese => vec![
            "右键点击 Steam 快捷方式，选择「属性」".to_string(),
            "在「目标」栏末尾添加：-cef-enable-debugging".to_string(),
            "点击「确定」保存设置".to_string(),
            "启动 Steam".to_string(),
        ],
        crate::i18n::Language::English => vec![
            "Right-click Steam shortcut, select 'Properties'".to_string(),
            "Add to end of 'Target' field: -cef-enable-debugging".to_string(),
            "Click 'OK' to save settings".to_string(),
            "Launch Steam".to_string(),
        ],
    };
    GuideDialog::new(GuideDialogType::ManualOperation {
        title: i18n.manual_restart_windows_title().to_string(),
        steps,
    })
}

#[cfg(target_os = "linux")]
pub fn create_linux_manual_guide(i18n: &I18n) -> GuideDialog {
    let steps = match i18n.language() {
        crate::i18n::Language::Chinese => vec![
            "关闭 Steam（如果正在运行）".to_string(),
            "打开终端".to_string(),
            "执行命令：steam -cef-enable-debugging &".to_string(),
            "或修改 Steam 快捷方式，在 Exec 行末尾添加 -cef-enable-debugging".to_string(),
        ],
        crate::i18n::Language::English => vec![
            "Close Steam (if running)".to_string(),
            "Open terminal".to_string(),
            "Execute: steam -cef-enable-debugging &".to_string(),
            "Or modify Steam shortcut, add -cef-enable-debugging to end of Exec line".to_string(),
        ],
    };
    GuideDialog::new(GuideDialogType::ManualOperation {
        title: i18n.manual_restart_linux_title().to_string(),
        steps,
    })
}
