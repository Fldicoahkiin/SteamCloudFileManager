use crate::error::{AppError, AppResult};
use crate::game_scanner::CloudGameInfo;
use crate::steam_api::{CloudFile, SteamCloudManager};
use crate::vdf_parser::{UserInfo, VdfParser};
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct SteamCloudApp {
    steam_manager: Arc<Mutex<SteamCloudManager>>,
    app_id_input: String,
    files: Vec<CloudFile>,
    selected_files: Vec<usize>,
    quota_info: Option<(u64, u64)>,
    status_message: String,
    is_connected: bool,
    show_error: bool,
    error_message: String,
    is_refreshing: bool,
    is_connecting: bool,
    remote_ready: bool,
    loader_rx: Option<Receiver<Result<Vec<CloudFile>, String>>>,
    connect_rx: Option<Receiver<Result<u32, String>>>,
    since_connected: Option<Instant>,
    local_save_paths: Vec<(String, PathBuf)>,
    cloud_games: Vec<CloudGameInfo>,
    show_game_selector: bool,
    is_scanning_games: bool,
    scan_games_rx: Option<Receiver<Result<crate::game_scanner::ScanResult, String>>>,
    vdf_parser: Option<VdfParser>,
    all_users: Vec<UserInfo>,
    show_user_selector: bool,
    show_about: bool,
    show_debug_warning: bool,
    about_icon_texture: Option<egui::TextureHandle>,
    guide_dialog: Option<crate::ui::GuideDialog>,
    restart_rx: Option<Receiver<crate::steam_process::RestartStatus>>,
    file_tree: Option<crate::file_tree::FileTree>,
    search_query: String,
    show_only_local: bool,
    show_only_cloud: bool,
    last_selected_index: Option<usize>,
}

impl SteamCloudApp {
    fn handle_error(&mut self, error: AppError) {
        tracing::error!(error = ?error, "操作失败");
        self.show_error(&error.to_string());
    }

    fn ensure_vdf_parser(&mut self) -> Option<&VdfParser> {
        if self.vdf_parser.is_none() {
            self.vdf_parser = VdfParser::new().ok();
        }
        self.vdf_parser.as_ref()
    }

    fn ensure_connected(&self) -> AppResult<()> {
        if !self.is_connected {
            return Err(AppError::SteamNotConnected);
        }
        Ok(())
    }

    fn validate_app_id(&self) -> AppResult<u32> {
        self.app_id_input
            .trim()
            .parse::<u32>()
            .map_err(|_| AppError::InvalidAppId)
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 设置字体
        crate::ui::setup_fonts(&cc.egui_ctx);

        let mut app = Self {
            steam_manager: Arc::new(Mutex::new(SteamCloudManager::new())),
            app_id_input: String::new(),
            files: Vec::new(),
            selected_files: Vec::new(),
            quota_info: None,
            status_message: "请输入App ID并连接到Steam".to_string(),
            is_connected: false,
            show_error: false,
            error_message: String::new(),
            is_refreshing: false,
            is_connecting: false,
            remote_ready: false,
            loader_rx: None,
            connect_rx: None,
            since_connected: None,
            local_save_paths: Vec::new(),
            cloud_games: Vec::new(),
            show_game_selector: false,
            is_scanning_games: false,
            scan_games_rx: None,
            vdf_parser: VdfParser::new().ok(),
            all_users: Vec::new(),
            show_user_selector: false,
            show_about: false,
            show_debug_warning: !crate::cdp_client::CdpClient::is_cdp_running(),
            about_icon_texture: None,
            guide_dialog: None,
            restart_rx: None,
            file_tree: None,
            search_query: String::new(),
            show_only_local: false,
            show_only_cloud: false,
            last_selected_index: None,
        };

        // 启动时自动扫描游戏
        app.scan_cloud_games();

        app
    }

    fn connect_to_steam(&mut self) {
        if self.app_id_input.trim().is_empty() {
            self.show_error("请输入App ID");
            return;
        }

        if self.is_connecting || self.connect_rx.is_some() {
            tracing::warn!("正在连接中，请勿重复点击");
            return;
        }

        let app_id = match self.validate_app_id() {
            Ok(id) => id,
            Err(e) => {
                self.handle_error(e);
                return;
            }
        };

        tracing::info!(app_id = app_id, "开始连接到 Steam");
        self.is_connecting = true;
        self.is_connected = false;
        self.remote_ready = false;
        self.files.clear();
        self.selected_files.clear();
        self.quota_info = None;
        self.status_message = format!("正在连接到 Steam (App ID: {})...", app_id);

        let steam_manager = self.steam_manager.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.connect_rx = Some(rx);

        std::thread::spawn(move || {
            let result = {
                let mut manager = match steam_manager.lock() {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::error!(error = %e, "Steam 管理器锁错误");
                        let _ = tx.send(Err("Steam 管理器锁错误".to_string()));
                        return;
                    }
                };
                manager.connect(app_id)
            };
            let _ = tx.send(result.map(|_| app_id).map_err(|e| e.to_string()));
        });
    }

    fn disconnect_from_steam(&mut self) {
        let mut manager = self.steam_manager.lock().expect("steam_manager 锁不可用");
        manager.disconnect();

        self.is_connected = false;
        self.is_connecting = false;
        self.remote_ready = false;
        self.files.clear();
        self.selected_files.clear();
        self.quota_info = None;
        self.since_connected = None;
        self.status_message = "已断开连接".to_string();
    }

    fn refresh_files(&mut self) {
        if let Err(e) = self.ensure_connected() {
            self.handle_error(e);
            return;
        }

        if self.loader_rx.is_some() {
            tracing::debug!("正在刷新中，跳过重复请求");
            return;
        }

        tracing::info!("开始刷新云文件列表");
        self.is_refreshing = true;
        self.files.clear();

        let steam_manager = self.steam_manager.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.loader_rx = Some(rx);

        let app_id = self.app_id_input.trim().parse::<u32>().unwrap_or(0);

        std::thread::spawn(move || {
            // 使用 FileService 统一获取文件
            let file_service = crate::file_manager::FileService::with_steam_manager(steam_manager);

            let files = match file_service.get_cloud_files(app_id) {
                Ok(files) => {
                    // CDP 数据合并
                    if app_id > 0 {
                        file_service
                            .merge_cdp_files(files, app_id)
                            .unwrap_or_else(|_| Vec::new())
                    } else {
                        files
                    }
                }
                Err(e) => {
                    tracing::error!("获取文件列表失败: {}", e);
                    Vec::new()
                }
            };

            let _ = tx.send(Ok(files));
        });
    }

    fn update_quota(&mut self) {
        if let Ok(manager) = self.steam_manager.lock() {
            if let Ok((total, available)) = manager.get_quota() {
                self.quota_info = Some((total, available));
            }
        }
    }

    fn download_selected_file(&mut self) {
        if self.selected_files.is_empty() {
            self.show_error("请选择要下载的文件");
            return;
        }

        // 使用批量下载，保持文件夹结构
        match crate::file_manager::batch_download_files_with_dialog(
            &self.files,
            &self.selected_files,
            self.steam_manager.clone(),
        ) {
            Ok(Some((success_count, failed_files))) => {
                if failed_files.is_empty() {
                    self.status_message = format!("成功下载 {} 个文件", success_count);
                } else {
                    let error_msg = format!(
                        "下载完成：成功 {} 个，失败 {} 个\n失败文件：{}",
                        success_count,
                        failed_files.len(),
                        failed_files.join(", ")
                    );
                    self.show_error(&error_msg);
                }
            }
            Ok(None) => {
                // 用户取消
            }
            Err(e) => {
                self.show_error(&format!("下载失败: {}", e));
            }
        }
    }

    fn upload_file(&mut self) {
        use crate::file_manager::FileOperationResult;

        let result = crate::file_manager::upload_file_coordinated(
            self.is_connected,
            self.steam_manager.clone(),
        );

        match result {
            FileOperationResult::Success(msg) => {
                self.status_message = msg;
            }
            FileOperationResult::SuccessWithRefresh(msg) => {
                self.status_message = msg;
                self.refresh_files();
            }
            FileOperationResult::Error(msg) => {
                self.show_error(&msg);
            }
        }
    }

    fn forget_selected_files(&mut self) {
        use crate::file_manager::FileOperationResult;

        let result = crate::file_manager::forget_selected_files_coordinated(
            &self.files,
            &self.selected_files,
            self.steam_manager.clone(),
        );

        match result {
            FileOperationResult::Success(msg) => {
                self.status_message = msg;
            }
            FileOperationResult::SuccessWithRefresh(msg) => {
                self.status_message = msg;
                self.refresh_files();
            }
            FileOperationResult::Error(msg) => {
                self.show_error(&msg);
            }
        }
    }

    fn delete_selected_files(&mut self) {
        use crate::file_manager::FileOperationResult;

        let result = crate::file_manager::delete_selected_files_coordinated(
            &self.files,
            &self.selected_files,
            self.steam_manager.clone(),
        );

        match result {
            FileOperationResult::Success(msg) => {
                self.status_message = msg;
            }
            FileOperationResult::SuccessWithRefresh(msg) => {
                self.status_message = msg;
                self.refresh_files();
            }
            FileOperationResult::Error(msg) => {
                self.show_error(&msg);
            }
        }
    }

    fn show_error(&mut self, message: &str) {
        self.error_message = message.to_string();
        self.show_error = true;
    }

    fn scan_cloud_games(&mut self) {
        // 先获取必要的数据，避免借用冲突
        let parser_data = self
            .ensure_vdf_parser()
            .map(|p| (p.get_steam_path().clone(), p.get_user_id().to_string()));

        if let Some((steam_path, user_id)) = parser_data {
            // 在扫描前强制跳转到 Steam 云存储页面
            let steam_url = "steam://openurl/https://store.steampowered.com/account/remotestorage";
            #[cfg(target_os = "macos")]
            let open_result = std::process::Command::new("open").arg(steam_url).spawn();
            #[cfg(target_os = "windows")]
            let open_result = std::process::Command::new("cmd")
                .args(["/C", "start", "", steam_url])
                .spawn();
            #[cfg(target_os = "linux")]
            let open_result = std::process::Command::new("xdg-open")
                .arg(steam_url)
                .spawn();

            match open_result {
                Ok(_) => tracing::info!("已请求 Steam 打开云存储页面"),
                Err(e) => tracing::warn!("无法打开 Steam 云存储页面: {}", e),
            }

            self.is_scanning_games = true;
            self.status_message = "正在扫描游戏库...".to_string();
            let (tx, rx) = std::sync::mpsc::channel();
            self.scan_games_rx = Some(rx);

            std::thread::spawn(move || {
                let result = crate::game_scanner::fetch_and_merge_games(steam_path, user_id)
                    .map_err(|e| e.to_string());
                let _ = tx.send(result);
            });
        } else {
            self.show_error("VDF 解析器未初始化");
        }
    }

    fn handle_file_drop(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        // 处理文件拖放
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        self.upload_file_from_path(path);
                    }
                }
            }
        });
    }

    fn upload_file_from_path(&mut self, path: &std::path::Path) {
        use crate::file_manager::FileOperationResult;

        let result = crate::file_manager::upload_file_from_path_coordinated(
            path,
            self.steam_manager.clone(),
        );

        match result {
            FileOperationResult::Success(msg) => {
                self.status_message = msg;
            }
            FileOperationResult::SuccessWithRefresh(msg) => {
                self.status_message = msg;
                self.refresh_files();
            }
            FileOperationResult::Error(msg) => {
                self.show_error(&msg);
            }
        }
    }

    fn draw_connection_panel(&mut self, ui: &mut egui::Ui) {
        if self.show_debug_warning {
            let (restart_clicked, dismiss_clicked) = crate::ui::draw_debug_warning_ui(ui);

            // 处理重启操作
            if restart_clicked {
                tracing::info!("用户点击自动重启 Steam");

                // 显示进度对话框
                self.guide_dialog = Some(crate::ui::create_restart_progress_dialog(
                    "正在关闭 Steam...".to_string(),
                ));

                // 启动异步重启
                let (tx, rx) = std::sync::mpsc::channel();
                self.restart_rx = Some(rx);

                let ctx = ui.ctx().clone();
                std::thread::spawn(move || {
                    crate::steam_process::restart_steam_with_status(tx, move || {
                        ctx.request_repaint();
                    });
                });

                self.show_debug_warning = false;
            }

            // 处理忽略操作
            if dismiss_clicked {
                tracing::info!("用户选择暂时忽略 CDP 调试警告");
                self.show_debug_warning = false;
            }
        }

        ui.horizontal(|ui| {
            let user_id = self.vdf_parser.as_ref().map(|p| p.get_user_id());

            crate::ui::draw_toolbar_buttons(
                ui,
                user_id,
                &mut self.show_about,
                &mut self.show_user_selector,
                &mut self.show_game_selector,
            );

            if self.show_user_selector && self.all_users.is_empty() {
                if let Some(parser) = &self.vdf_parser {
                    if let Ok(users) = parser.get_all_users_info() {
                        self.all_users = users;
                    }
                }
            }

            if self.show_game_selector
                && !self.is_scanning_games
                && self.scan_games_rx.is_none()
                && self.cloud_games.is_empty()
            {
                self.scan_cloud_games();
            }

            let action = crate::ui::draw_connection_controls(
                ui,
                &mut self.app_id_input,
                self.is_connected,
                self.is_connecting,
            );

            match action {
                crate::ui::ConnectionAction::InputChanged => {
                    self.is_connected = false;
                    self.remote_ready = false;
                    self.disconnect_from_steam();
                }
                crate::ui::ConnectionAction::Connect => {
                    self.connect_to_steam();
                }
                crate::ui::ConnectionAction::Disconnect => {
                    self.disconnect_from_steam();
                }
                crate::ui::ConnectionAction::None => {}
            }
        });
    }

    fn draw_file_list(&mut self, ui: &mut egui::Ui) {
        // 树状视图
        if let Some(tree) = &mut self.file_tree {
            let mut state = crate::ui::TreeViewState {
                search_query: &mut self.search_query,
                show_only_local: &mut self.show_only_local,
                show_only_cloud: &mut self.show_only_cloud,
                last_selected_index: &mut self.last_selected_index,
            };
            crate::ui::render_file_tree(
                ui,
                tree,
                &mut self.selected_files,
                &self.files,
                &self.local_save_paths,
                self.remote_ready,
                &mut state,
            );
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("没有找到云文件");
            });
        }
    }

    fn draw_action_buttons(&mut self, ui: &mut egui::Ui) {
        ui.separator();

        let can_ops =
            self.is_connected && self.remote_ready && !self.is_refreshing && !self.is_connecting;

        let has_selection = !self.selected_files.is_empty();
        let selected_count = self.selected_files.len();
        let total_count = self.files.len();

        let selected_total_size: u64 = self
            .selected_files
            .iter()
            .filter_map(|&idx| self.files.get(idx))
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

        match action {
            crate::ui::FileAction::SelectAll => {
                self.selected_files = crate::ui::select_all_files(self.files.len());
            }
            crate::ui::FileAction::InvertSelection => {
                self.selected_files =
                    crate::ui::invert_file_selection(&self.selected_files, self.files.len());
            }
            crate::ui::FileAction::ClearSelection => {
                self.selected_files = crate::ui::clear_file_selection();
            }
            crate::ui::FileAction::DownloadSelected => {
                self.download_selected_file();
            }
            crate::ui::FileAction::UploadFile => {
                self.upload_file();
            }
            crate::ui::FileAction::DeleteSelected => {
                self.delete_selected_files();
            }
            crate::ui::FileAction::ForgetSelected => {
                self.forget_selected_files();
            }
            crate::ui::FileAction::None => {}
        }
    }

    fn draw_status_panel(&mut self, ui: &mut egui::Ui) {
        let cloud_enabled = if self.is_connected {
            self.steam_manager
                .lock()
                .ok()
                .and_then(|m| m.is_cloud_enabled_for_app().ok())
        } else {
            None
        };

        let (account_enabled, app_enabled) = if self.is_connected && self.remote_ready {
            if let Ok(manager) = self.steam_manager.lock() {
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
            status_message: self.status_message.clone(),
            cloud_enabled,
            is_connected: self.is_connected,
            remote_ready: self.remote_ready,
            account_enabled,
            app_enabled,
            quota_info: self.quota_info,
        };

        let action = crate::ui::draw_complete_status_panel(ui, &state);

        match action {
            crate::ui::StatusPanelAction::ToggleCloudEnabled => {
                if let Ok(manager) = self.steam_manager.lock() {
                    if let Ok(enabled) = manager.is_cloud_enabled_for_app() {
                        let _ = manager.set_cloud_enabled_for_app(!enabled);
                    }
                }
            }
            crate::ui::StatusPanelAction::None => {}
        }
    }
}

impl eframe::App for SteamCloudApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_connected {
            if let Ok(manager) = self.steam_manager.try_lock() {
                manager.run_callbacks();
            }

            if !self.remote_ready && !self.is_refreshing {
                if let Some(since) = self.since_connected {
                    if since.elapsed() >= Duration::from_secs(2) {
                        tracing::info!("Steam API已准备就绪");
                        self.refresh_files();
                        self.remote_ready = true;
                    }
                }
            }
        }

        if let Some(rx) = &self.connect_rx {
            match rx.try_recv() {
                Ok(Ok(app_id)) => {
                    self.is_connecting = false;
                    self.is_connected = true;
                    self.status_message = format!("已连接到Steam (App ID: {})", app_id);
                    self.since_connected = Some(Instant::now());
                    self.connect_rx = None;
                    tracing::info!("Steam连接成功");
                }
                Ok(Err(err)) => {
                    self.is_connecting = false;
                    self.connect_rx = None;
                    self.show_error(&format!("连接Steam失败: {}", err));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_connecting = false;
                    self.connect_rx = None;
                }
            }
        }

        if let Some(rx) = &self.loader_rx {
            match rx.try_recv() {
                Ok(Ok(files)) => {
                    let count = files.len();
                    self.files = files;
                    self.selected_files.clear();
                    self.update_quota();

                    // 更新本地存档路径
                    if let Ok(app_id) = self.app_id_input.parse::<u32>() {
                        let parser_data = self
                            .ensure_vdf_parser()
                            .map(|p| (p.get_steam_path().clone(), p.get_user_id().to_string()));

                        if let Some((steam_path, user_id)) = parser_data {
                            self.local_save_paths = crate::path_resolver::collect_local_save_paths(
                                &self.files,
                                &steam_path,
                                &user_id,
                                app_id,
                            );
                        } else {
                            self.local_save_paths.clear();
                        }
                    } else {
                        self.local_save_paths.clear();
                    }

                    // 构建文件树
                    self.file_tree = Some(crate::file_tree::FileTree::new(&self.files));

                    self.status_message = format!("已加载 {} 个文件", count);
                    self.is_refreshing = false;
                    self.remote_ready = true;
                    self.loader_rx = None;
                }
                Ok(Err(err)) => {
                    self.show_error(&format!("刷新文件列表失败: {}", err));
                    self.is_refreshing = false;
                    self.loader_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_refreshing = false;
                    self.loader_rx = None;
                }
            }
        }

        if let Some(rx) = &self.scan_games_rx {
            match rx.try_recv() {
                Ok(Ok(result)) => {
                    self.cloud_games = result.games;

                    // 构建详细的状态信息
                    let mut status_parts = Vec::new();
                    status_parts.push(format!("VDF: {} 个", result.vdf_count));
                    if result.cdp_count > 0 {
                        status_parts.push(format!("CDP: {} 个", result.cdp_count));
                    }
                    status_parts.push(format!("总计: {} 个游戏", self.cloud_games.len()));

                    self.status_message = status_parts.join(" | ");

                    // 如果 CDP 获取为 0，弹出警告
                    if result.cdp_count == 0 && crate::cdp_client::CdpClient::is_cdp_running() {
                        self.show_error(
                            "CDP 未获取到游戏数据！\n\n可能原因：\n\
                            1. Steam 客户端未响应跳转请求\n\
                            2. 页面加载未完成\n\
                            3. 未登录 Steam 网页\n\n\
                        ",
                        );
                    }

                    self.is_scanning_games = false;
                    self.scan_games_rx = None;
                }
                Ok(Err(err)) => {
                    self.show_error(&format!("扫描游戏失败: {}", err));
                    self.is_scanning_games = false;
                    self.scan_games_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_scanning_games = false;
                    self.scan_games_rx = None;
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Steam 云文件管理器");
            self.draw_connection_panel(ui);
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            self.draw_action_buttons(ui);
            self.draw_status_panel(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_connected && self.remote_ready {
                self.handle_file_drop(ctx, ui);
            }

            self.draw_file_list(ui);
        });

        if self.show_error
            && crate::ui::draw_error_window(ctx, &mut self.show_error, &self.error_message)
        {
            self.show_error = false;
        }

        if self.show_game_selector {
            let (selected_app_id, refresh_clicked) = crate::ui::draw_game_selector_window(
                ctx,
                &mut self.show_game_selector,
                &self.cloud_games,
                self.is_scanning_games,
            );
            if let Some(app_id) = selected_app_id {
                self.app_id_input = app_id.to_string();
                self.show_game_selector = false;
                self.connect_to_steam();
            }
            if refresh_clicked {
                tracing::info!("用户点击刷新游戏库");
                self.scan_cloud_games();
            }
        }

        // 处理 Steam 重启状态更新
        if let Some(rx) = &self.restart_rx {
            if let Ok(status) = rx.try_recv() {
                use crate::steam_process::RestartStatus;
                match status {
                    RestartStatus::Closing => {
                        if let Some(dialog) = &mut self.guide_dialog {
                            dialog.update_status("正在关闭 Steam...".to_string(), false, false);
                        }
                    }
                    RestartStatus::Starting => {
                        if let Some(dialog) = &mut self.guide_dialog {
                            dialog.update_status("正在启动 Steam...".to_string(), false, false);
                        }
                    }
                    RestartStatus::Success => {
                        if let Some(dialog) = &mut self.guide_dialog {
                            dialog.update_status("Steam 已成功重启!".to_string(), true, false);
                        }
                        self.restart_rx = None;
                    }
                    RestartStatus::Error(msg) => {
                        tracing::error!("Steam 重启失败: {}", msg);
                        self.restart_rx = None;

                        // 显示手动操作引导
                        #[cfg(target_os = "macos")]
                        {
                            self.guide_dialog = Some(crate::ui::create_macos_manual_guide());
                        }
                        #[cfg(target_os = "windows")]
                        {
                            self.guide_dialog = Some(crate::ui::create_windows_manual_guide());
                        }
                        #[cfg(target_os = "linux")]
                        {
                            self.guide_dialog = Some(crate::ui::create_linux_manual_guide());
                        }
                    }
                }
            }
        }

        // 绘制引导对话框
        let mut close_dialog = false;
        if let Some(dialog) = &mut self.guide_dialog {
            let action = dialog.draw(ctx);
            match action {
                crate::ui::GuideDialogAction::Confirm => {
                    tracing::info!("用户确认引导对话框");
                    close_dialog = true;
                }
                crate::ui::GuideDialogAction::None => {}
            }
            if !dialog.show {
                close_dialog = true;
            }
        }
        if close_dialog {
            self.guide_dialog = None;
        }

        if self.show_user_selector {
            let selected_user_id = crate::ui::draw_user_selector_window(
                ctx,
                &mut self.show_user_selector,
                &self.all_users,
            );
            if let Some(user_id) = selected_user_id {
                if let Some(parser) = &self.vdf_parser {
                    let steam_path = parser.get_steam_path().clone();
                    self.vdf_parser = Some(VdfParser::with_user_id(steam_path, user_id));
                    self.cloud_games.clear();
                    self.status_message = "已切换用户".to_string();
                    self.scan_cloud_games();
                }
                self.show_user_selector = false;
            }
        }

        if self.show_about {
            crate::ui::draw_about_window(ctx, &mut self.show_about, &mut self.about_icon_texture);
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
