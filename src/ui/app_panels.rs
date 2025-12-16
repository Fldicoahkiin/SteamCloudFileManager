use crate::app_state::{ConnectionState, DialogState, FileListState, GameLibraryState, MiscState};
use crate::async_handlers::AsyncHandlers;
use crate::steam_api::SteamCloudManager;
use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TopPanelEvent {
    None,
    ScanGames,
    Connect,
    Disconnect,
    Refresh,
    Restart,
}

// 顶部面板渲染
pub fn render_top_panel(
    ui: &mut egui::Ui,
    dialogs: &mut DialogState,
    connection: &mut ConnectionState,
    game_library: &mut GameLibraryState,
    async_handlers: &mut AsyncHandlers,
) -> TopPanelEvent {
    ui.heading("Steam 云文件管理器");

    let mut event = TopPanelEvent::None;

    // 调试警告
    if dialogs.show_debug_warning {
        let (restart_clicked, dismiss_clicked, show_manual) = crate::ui::draw_debug_warning_ui(ui);

        if show_manual {
            dialogs.guide_dialog = Some(crate::ui::get_manual_guide_dialog());
        }

        if restart_clicked {
            dialogs.guide_dialog = Some(crate::ui::create_restart_progress_dialog(
                "正在关闭 Steam...".to_string(),
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
        crate::ui::draw_toolbar_buttons(
            ui,
            None,
            &mut dialogs.show_about,
            &mut game_library.show_user_selector,
            &mut game_library.show_game_selector,
        );

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BottomPanelEvent {
    None,
    SelectAll,
    InvertSelection,
    ClearSelection,
    Download,
    Upload,
    Delete,
    Forget,
    ToggleCloud,
}

// 底部面板渲染
pub fn render_bottom_panel(
    ui: &mut egui::Ui,
    connection: &ConnectionState,
    file_list: &mut FileListState,
    misc: &MiscState,
    steam_manager: &Arc<Mutex<SteamCloudManager>>,
) -> BottomPanelEvent {
    // 文件操作按钮
    ui.separator();

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

    let action = crate::ui::draw_file_action_buttons(
        ui,
        can_ops,
        has_selection,
        selected_count,
        total_count,
        selected_total_size,
    );

    let mut event = match action {
        crate::ui::FileAction::SelectAll => BottomPanelEvent::SelectAll,
        crate::ui::FileAction::InvertSelection => BottomPanelEvent::InvertSelection,
        crate::ui::FileAction::ClearSelection => BottomPanelEvent::ClearSelection,
        crate::ui::FileAction::DownloadSelected => BottomPanelEvent::Download,
        crate::ui::FileAction::Upload => BottomPanelEvent::Upload,
        crate::ui::FileAction::DeleteSelected => BottomPanelEvent::Delete,
        crate::ui::FileAction::ForgetSelected => BottomPanelEvent::Forget,
        crate::ui::FileAction::None => BottomPanelEvent::None,
    };

    // 状态面板
    let cloud_enabled = if connection.is_connected {
        steam_manager
            .lock()
            .ok()
            .and_then(|m| m.is_cloud_enabled_for_app().ok())
    } else {
        None
    };

    let (account_enabled, app_enabled) = if connection.is_connected && connection.remote_ready {
        if let Ok(manager) = steam_manager.lock() {
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

    let state = crate::ui::StatusPanelState {
        status_message: misc.status_message.clone(),
        cloud_enabled,
        is_connected: connection.is_connected,
        remote_ready: connection.remote_ready,
        account_enabled,
        app_enabled,
        quota_info: misc.quota_info,
    };

    let action = crate::ui::draw_complete_status_panel(ui, &state);

    if matches!(action, crate::ui::StatusPanelAction::ToggleCloudEnabled)
        && event == BottomPanelEvent::None
    {
        event = BottomPanelEvent::ToggleCloud;
    }

    event
}

// 中心面板渲染
pub fn render_center_panel(
    ui: &mut egui::Ui,
    connection: &ConnectionState,
    file_list: &mut FileListState,
) {
    // 文件列表
    if !connection.is_connected && !connection.is_connecting {
        crate::ui::draw_disconnected_view(ui);
    } else if connection.is_connecting || (connection.is_connected && !connection.remote_ready) {
        crate::ui::draw_loading_view(ui, connection.is_connecting);
    } else if let Some(tree) = &mut file_list.file_tree {
        let mut state = crate::ui::TreeViewState {
            search_query: &mut file_list.search_query,
            show_only_local: &mut file_list.show_only_local,
            show_only_cloud: &mut file_list.show_only_cloud,
            last_selected_index: &mut file_list.last_selected_index,
        };
        crate::ui::render_file_tree(
            ui,
            tree,
            &mut file_list.selected_files,
            &file_list.files,
            &file_list.local_save_paths,
            connection.remote_ready,
            &mut state,
        );
    } else {
        crate::ui::draw_no_files_view(ui);
    }
}
