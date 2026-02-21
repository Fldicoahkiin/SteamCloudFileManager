use crate::app_state::{ConnectionState, DialogState, FileListState, GameLibraryState, MiscState};
use crate::async_handlers::AsyncHandlers;
use crate::i18n::I18n;
use crate::icons;
use crate::steam_worker::SteamWorkerManager;
use eframe::egui;
use std::sync::{Arc, Mutex};

// 文件操作类型
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

// 状态面板的用户操作
#[derive(Debug, Clone, PartialEq)]
pub enum StatusPanelAction {
    None,
    ToggleCloudEnabled,
    ShowAppInfo(u32),
    ShowSymlinkManager(u32),
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

// 顶部面板事件
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TopPanelEvent {
    None,
    ScanGames,
    Connect,
    Disconnect,
    Refresh,
    Restart,
}

// 底部面板事件
#[derive(Debug, Clone, PartialEq)]
pub enum BottomPanelEvent {
    None,
    SelectAll,
    InvertSelection,
    ClearSelection,
    Download,
    Upload,
    Delete,
    Forget,
    SyncToCloud,
    ToggleCloud,
    CompareFiles,
    ShowAppInfo(u32),
    ShowSymlinkManager(u32),
}

// 顶部面板渲染
pub fn render_top_panel(
    ui: &mut egui::Ui,
    dialogs: &mut DialogState,
    connection: &mut ConnectionState,
    game_library: &mut GameLibraryState,
    file_state: &FileListState,
    async_handlers: &mut AsyncHandlers,
    misc: &mut MiscState,
) -> TopPanelEvent {
    ui.horizontal(|ui| {
        ui.heading(misc.i18n.app_title());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let current_lang = misc.i18n.language();
            egui::ComboBox::from_id_salt("language_selector")
                .selected_text(current_lang.display_name())
                .width(80.0)
                .show_ui(ui, |ui| {
                    for lang in crate::i18n::Language::all() {
                        let is_selected = current_lang == *lang;
                        if ui
                            .selectable_label(is_selected, lang.display_name())
                            .clicked()
                        {
                            misc.i18n.set_language(*lang);
                            // 持久化语言设置
                            if let Err(e) = crate::config::update_config(|config| {
                                config.appearance.language = lang.to_config().to_string();
                            }) {
                                tracing::error!("保存语言设置失败: {}", e);
                            }
                        }
                    }
                });
            // 在语言选择器左边显示标签
            ui.label(misc.i18n.language_label());
        });
    });

    let mut event = TopPanelEvent::None;

    // 调试警告
    if dialogs.show_debug_warning {
        let (restart_clicked, dismiss_clicked, show_manual) =
            crate::ui::draw_debug_warning_ui(ui, &misc.i18n);

        if show_manual {
            dialogs.guide_dialog = Some(crate::ui::get_manual_guide_dialog(&misc.i18n));
        }

        if restart_clicked {
            dialogs.guide_dialog = Some(crate::ui::create_restart_progress_dialog(
                misc.i18n.closing_steam().to_string(),
            ));
            event = TopPanelEvent::Restart;
            dialogs.show_debug_warning = false;
        }

        if dismiss_clicked {
            dialogs.show_debug_warning = false;
        }
    }

    // 工具栏和连接控制
    ui.horizontal(|ui| {
        let mut toolbar_state = crate::ui::ToolbarState {
            user_id: None,
            has_files: !file_state.files.is_empty(),
            on_settings: &mut dialogs.show_settings,
            on_user_selector: &mut game_library.show_user_selector,
            on_game_selector: &mut game_library.show_game_selector,
            on_backup: &mut dialogs.show_backup,
        };
        crate::ui::draw_toolbar_buttons(ui, &mut toolbar_state, &misc.i18n);

        if game_library.show_game_selector
            && !game_library.is_scanning_games
            && async_handlers.scan_games_rx.is_none()
            && game_library.cloud_games.is_empty()
            && event == TopPanelEvent::None
        {
            event = TopPanelEvent::ScanGames;
        }

        let action = crate::ui::draw_connection_controls(
            ui,
            &mut connection.app_id_input,
            connection.is_connected,
            connection.is_connecting,
            &misc.i18n,
        );

        match action {
            crate::ui::ConnectionAction::InputChanged => {
                connection.is_connected = false;
                connection.remote_ready = false;
                if event == TopPanelEvent::None {
                    event = TopPanelEvent::Disconnect;
                }
            }
            crate::ui::ConnectionAction::Connect => {
                if event == TopPanelEvent::None {
                    event = TopPanelEvent::Connect;
                }
            }
            crate::ui::ConnectionAction::Disconnect => {
                if event == TopPanelEvent::None {
                    event = TopPanelEvent::Disconnect;
                }
            }
            crate::ui::ConnectionAction::Refresh => {
                if event == TopPanelEvent::None {
                    event = TopPanelEvent::Refresh;
                }
            }
            crate::ui::ConnectionAction::None => {}
        }
    });

    event
}

// 底部面板渲染
pub fn render_bottom_panel(
    ui: &mut egui::Ui,
    connection: &ConnectionState,
    file_list: &mut FileListState,
    misc: &MiscState,
    _game_library: &GameLibraryState,
    steam_manager: &Arc<Mutex<SteamWorkerManager>>,
) -> BottomPanelEvent {
    // 文件操作按钮
    let can_ops = connection.is_connected
        && connection.remote_ready
        && !file_list.is_refreshing
        && !connection.is_connecting;

    let has_selection = !file_list.selected_files.is_empty();
    let selected_count = file_list.selected_files.len();
    let total_count = file_list.files.len();

    let selected_total_size: u64 = file_list
        .selected_files
        .iter()
        .filter_map(|&idx| file_list.files.get(idx))
        .map(|f| f.size)
        .sum();

    let action = draw_file_action_buttons(
        ui,
        can_ops,
        has_selection,
        selected_count,
        total_count,
        selected_total_size,
        &misc.i18n,
    );

    let mut event = match action {
        FileAction::SelectAll => BottomPanelEvent::SelectAll,
        FileAction::InvertSelection => BottomPanelEvent::InvertSelection,
        FileAction::ClearSelection => BottomPanelEvent::ClearSelection,
        FileAction::DownloadSelected => BottomPanelEvent::Download,
        FileAction::Upload => BottomPanelEvent::Upload,
        FileAction::DeleteSelected => BottomPanelEvent::Delete,
        FileAction::ForgetSelected => BottomPanelEvent::Forget,
        FileAction::SyncToCloud => BottomPanelEvent::SyncToCloud,
        FileAction::CompareFiles => BottomPanelEvent::CompareFiles,
        FileAction::None => BottomPanelEvent::None,
    };

    // 状态面板
    let cloud_enabled = if connection.is_connected {
        steam_manager
            .lock()
            .ok()
            .and_then(|mut m| m.is_cloud_enabled_for_app().ok())
    } else {
        None
    };

    let (account_enabled, app_enabled) = if connection.is_connected && connection.remote_ready {
        if let Ok(mut manager) = steam_manager.lock() {
            (
                manager.is_cloud_enabled_for_account().ok(),
                manager.is_cloud_enabled_for_app().ok(),
            )
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let app_id = connection.app_id_input.parse::<u32>().unwrap_or(0);

    let state = StatusPanelState {
        status_message: misc.status_message.clone(),
        cloud_enabled,
        is_connected: connection.is_connected,
        remote_ready: connection.remote_ready,
        account_enabled,
        app_enabled,
        quota_info: misc.quota_info,
        app_id,
    };

    let action = draw_complete_status_panel(ui, &state, &misc.i18n);

    match action {
        StatusPanelAction::ToggleCloudEnabled if event == BottomPanelEvent::None => {
            event = BottomPanelEvent::ToggleCloud;
        }
        StatusPanelAction::ShowAppInfo(id) if event == BottomPanelEvent::None => {
            event = BottomPanelEvent::ShowAppInfo(id);
        }
        StatusPanelAction::ShowSymlinkManager(id) if event == BottomPanelEvent::None => {
            event = BottomPanelEvent::ShowSymlinkManager(id);
        }
        _ => {}
    }

    event
}

// 绘制文件操作按钮栏
fn draw_file_action_buttons(
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

// 绘制完整的状态面板
fn draw_complete_status_panel(
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
                ui.label(i18n.cloud_status_not_ready());
            });
        }
    }

    // 配额信息
    if let Some((total, available)) = state.quota_info {
        draw_quota_info(ui, total, available, i18n);
    }

    // 显示 appinfo.vdf 按钮 和 软链接管理按钮
    if state.is_connected && state.app_id > 0 {
        ui.horizontal(|ui| {
            if ui.button(i18n.show_appinfo_vdf()).clicked() {
                action = StatusPanelAction::ShowAppInfo(state.app_id);
            }
            if ui.button(i18n.symlink_title()).clicked() {
                action = StatusPanelAction::ShowSymlinkManager(state.app_id);
            }
        });
    }

    action
}

// 绘制状态消息栏
fn draw_status_message(
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

// 绘制云存储状态信息
fn draw_cloud_status(
    ui: &mut egui::Ui,
    account_enabled: Option<bool>,
    _app_enabled: Option<bool>,
    i18n: &I18n,
) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", i18n.account_cloud_status()));
        match account_enabled {
            Some(true) => ui.label(format!("{} {}", icons::CHECK, i18n.logged_in())),
            Some(false) => ui.label(format!("{} {}", icons::CLOSE, i18n.not_logged_in())),
            None => ui.label(format!("{} Unknown", icons::QUESTION)),
        };
    });
}

// 绘制配额信息
fn draw_quota_info(ui: &mut egui::Ui, total: u64, available: u64, i18n: &I18n) {
    ui.horizontal(|ui| {
        let used = total - available;
        let usage_percent = (used as f32 / total as f32 * 100.0).round();
        let used_str = crate::file_manager::format_size(used);
        let total_str = crate::file_manager::format_size(total);
        let text = i18n.quota_usage(usage_percent, &used_str, &total_str);
        ui.label(text);
    });
}

// 中心面板渲染
pub fn render_center_panel(
    ui: &mut egui::Ui,
    connection: &ConnectionState,
    file_list: &mut FileListState,
    misc: &MiscState,
) {
    // 文件列表
    if !connection.is_connected && !connection.is_connecting {
        draw_disconnected_view(ui, &misc.i18n);
    } else if connection.is_connecting || (connection.is_connected && !connection.remote_ready) {
        draw_loading_view(ui, connection.is_connecting, &misc.i18n);
    } else if let Some(tree) = &mut file_list.file_tree {
        let mut state = crate::ui::TreeViewState {
            search_query: &mut file_list.search_query,
            show_only_local: &mut file_list.show_only_local,
            show_only_cloud: &mut file_list.show_only_cloud,
            last_selected_index: &mut file_list.last_selected_index,
        };
        crate::ui::render_file_tree(
            ui,
            crate::ui::FileTreeRenderParams {
                tree,
                selected_files: &mut file_list.selected_files,
                local_save_paths: &file_list.local_save_paths,
                remote_ready: connection.remote_ready,
                state: &mut state,
                i18n: &misc.i18n,
                sync_status_map: &file_list.sync_status_map,
            },
        );
    } else {
        draw_no_files_view(ui, &misc.i18n);
    }
}

// 绘制未连接状态的提示
fn draw_disconnected_view(ui: &mut egui::Ui, i18n: &I18n) {
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
fn draw_loading_view(ui: &mut egui::Ui, is_connecting: bool, i18n: &I18n) {
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
fn draw_no_files_view(ui: &mut egui::Ui, i18n: &I18n) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 2.0 - 50.0);
        ui.heading(i18n.no_cloud_files());
        ui.add_space(10.0);
        ui.label(i18n.no_cloud_files_hint());
    });
}
