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
    show_upload_type_selector: bool,
    upload_preview: Option<crate::ui::UploadPreviewDialog>,
    upload_progress: Option<crate::ui::UploadProgressDialog>,
    upload_complete: Option<crate::ui::UploadCompleteDialog>,
    upload_rx: Option<Receiver<Result<String, String>>>,
    upload_progress_rx: Option<Receiver<(usize, usize, String)>>,
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
            show_upload_type_selector: false,
            upload_preview: None,
            upload_progress: None,
            upload_complete: None,
            upload_rx: None,
            upload_progress_rx: None,
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
        self.file_tree = None;
        self.local_save_paths.clear();
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
        self.file_tree = None; // 清空文件树
        self.local_save_paths.clear();
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

    // 上传
    fn upload(&mut self) {
        if !self.is_connected {
            self.show_error("未连接到 Steam");
            return;
        }

        // 显示选择对话框
        self.show_upload_type_selector = true;
    }

    // 开始上传
    fn start_upload(&mut self, mut queue: crate::file_manager::UploadQueue) {
        let total_files = queue.total_files();

        // 显示进度对话框
        self.upload_progress = Some(crate::ui::UploadProgressDialog::new(total_files));

        // 在后台线程执行上传
        let steam_manager = self.steam_manager.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let (progress_tx, progress_rx) = std::sync::mpsc::channel();
        self.upload_rx = Some(rx);
        self.upload_progress_rx = Some(progress_rx);

        std::thread::spawn(move || {
            let executor = crate::file_manager::UploadExecutor::new(steam_manager)
                .with_progress_callback(move |current, total, filename| {
                    let _ = progress_tx.send((current, total, filename.to_string()));
                });

            match executor.execute(&mut queue) {
                Ok(result) => {
                    // 使用 JSON 传递完整结果
                    let result_json = serde_json::json!({
                        "success_count": result.success_count,
                        "failed_count": result.failed_count,
                        "total_size": result.total_size,
                        "elapsed_secs": result.elapsed_secs,
                        "failed_files": result.failed_files,
                    });
                    let _ = tx.send(Ok(result_json.to_string()));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });
    }

    fn forget_selected_files(&mut self) {
        use crate::file_manager::FileOperationResult;

        let result = crate::file_manager::forget_selected_files_coordinated(
            &self.files,
            &self.selected_files,
            self.steam_manager.clone(),
        );

        match result {
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
        if !self.is_connected {
            self.show_error("未连接到 Steam");
            return;
        }

        // 使用新的上传系统
        let mut queue = crate::file_manager::UploadQueue::new();

        if path.is_file() {
            if let Err(e) = queue.add_file(path.to_path_buf()) {
                self.show_error(&format!("添加文件失败: {}", e));
                return;
            }
        } else if path.is_dir() {
            if let Err(e) = queue.add_folder(path) {
                self.show_error(&format!("添加文件夹失败: {}", e));
                return;
            }
        } else {
            self.show_error("无效的文件路径");
            return;
        }

        if queue.total_files() > 0 {
            // 直接开始上传，不显示预览
            self.start_upload(queue);
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
                crate::ui::ConnectionAction::Refresh => {
                    self.refresh_files();
                }
                crate::ui::ConnectionAction::None => {}
            }
        });
    }

    fn draw_file_list(&mut self, ui: &mut egui::Ui) {
        if !self.is_connected && !self.is_connecting {
            // 未连接状态
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 80.0);
                ui.heading("请输入 App ID 并连接到 Steam");
                ui.add_space(20.0);
                ui.label("您可以：");
                ui.label("点击上方的 '游戏库' 按钮选择游戏");
                ui.label("或直接输入 App ID 并点击 '连接'");
            });
        } else if self.is_connecting || (self.is_connected && !self.remote_ready) {
            // 连接中或加载中状态
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 40.0);
                ui.spinner();
                ui.add_space(10.0);
                if self.is_connecting {
                    ui.label("正在连接到 Steam...");
                } else {
                    ui.label("正在加载文件列表...");
                }
            });
        } else if let Some(tree) = &mut self.file_tree {
            // 已连接且有文件树
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
            // 已连接但没有文件
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 50.0);
                ui.heading("没有找到云文件");
                ui.add_space(10.0);
                ui.label("该游戏没有云存档文件");
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
            crate::ui::FileAction::Upload => {
                self.upload();
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

            // 检查超时（30秒）
            if !self.remote_ready && self.is_refreshing {
                if let Some(since) = self.since_connected {
                    if since.elapsed() >= Duration::from_secs(30) {
                        tracing::warn!("Steam API 加载超时，停止等待");
                        self.is_refreshing = false;
                        self.remote_ready = true;
                        self.loader_rx = None;
                        self.status_message = "加载超时，请重试".to_string();
                    }
                }
            }
        }

        if let Some(rx) = &self.connect_rx {
            match rx.try_recv() {
                Ok(Ok(app_id)) => {
                    self.is_connecting = false;
                    self.is_connected = true;
                    self.status_message = format!("正在加载文件列表 (App ID: {})...", app_id);
                    self.since_connected = Some(Instant::now());
                    self.connect_rx = None;
                    tracing::info!("Steam连接成功");

                    // 连接成功后立即开始刷新文件
                    self.refresh_files();
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

        // 处理批量上传进度更新
        if let Some(rx) = &self.upload_progress_rx {
            match rx.try_recv() {
                Ok((current, total, filename)) => {
                    if let Some(progress) = &mut self.upload_progress {
                        progress.current_index = current;
                        progress.total_files = total;
                        progress.current_file = filename.clone();
                        progress.progress = current as f32 / total as f32;

                        // 添加到已完成列表
                        if current > progress.completed_files.len() {
                            progress.completed_files.push(filename);
                        }
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.upload_progress_rx = None;
                }
            }
        }

        // 处理批量上传结果
        if let Some(rx) = &self.upload_rx {
            match rx.try_recv() {
                Ok(Ok(msg)) => {
                    // 解析 JSON 结果
                    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&msg) {
                        let success_count = result["success_count"].as_u64().unwrap_or(0) as usize;
                        let failed_count = result["failed_count"].as_u64().unwrap_or(0) as usize;
                        let total_size = result["total_size"].as_u64().unwrap_or(0);
                        let elapsed_secs = result["elapsed_secs"].as_u64().unwrap_or(0);

                        let failed_files: Vec<(String, String)> = result["failed_files"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|item| {
                                        let filename = item[0].as_str()?.to_string();
                                        let error = item[1].as_str()?.to_string();
                                        Some((filename, error))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        self.upload_progress = None;
                        self.upload_complete = Some(crate::ui::UploadCompleteDialog::new(
                            success_count,
                            failed_count,
                            total_size,
                            elapsed_secs,
                            failed_files,
                        ));
                    }
                    self.upload_rx = None;
                }
                Ok(Err(err)) => {
                    self.upload_progress = None;
                    self.show_error(&format!("上传失败: {}", err));
                    self.upload_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.upload_rx = None;
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

        // 上传类型选择器
        if self.show_upload_type_selector {
            egui::Window::new("📎 选择上传类型")
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label("请选择要上传的类型：");
                        ui.add_space(20.0);

                        if ui.button("📄 上传文件（可多选）").clicked() {
                            self.show_upload_type_selector = false;
                            match crate::file_manager::upload_files_with_dialog(
                                self.steam_manager.clone(),
                            ) {
                                Ok(Some(queue)) => {
                                    self.upload_preview =
                                        Some(crate::ui::UploadPreviewDialog::new(queue));
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    self.show_error(&format!("选择文件失败: {}", e));
                                }
                            }
                        }

                        ui.add_space(10.0);

                        if ui.button("📂 上传文件夹（递归）").clicked() {
                            self.show_upload_type_selector = false;
                            match crate::file_manager::upload_folder_with_dialog(
                                self.steam_manager.clone(),
                            ) {
                                Ok(Some(queue)) => {
                                    self.upload_preview =
                                        Some(crate::ui::UploadPreviewDialog::new(queue));
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    self.show_error(&format!("选择文件夹失败: {}", e));
                                }
                            }
                        }

                        ui.add_space(10.0);

                        if ui.button("✖ 取消").clicked() {
                            self.show_upload_type_selector = false;
                        }

                        ui.add_space(10.0);
                    });
                });
        }

        // 上传预览对话框
        if let Some(preview) = &mut self.upload_preview {
            match preview.draw(ctx) {
                crate::ui::UploadAction::Confirm => {
                    // 开始上传
                    if let Some(preview) = self.upload_preview.take() {
                        self.start_upload(preview.queue);
                    }
                }
                crate::ui::UploadAction::Cancel => {
                    self.upload_preview = None;
                }
                crate::ui::UploadAction::None => {}
            }
        }

        // 上传进度对话框
        if let Some(progress) = &mut self.upload_progress {
            progress.draw(ctx);
            if !progress.show {
                self.upload_progress = None;
            }
        }

        // 上传完成对话框
        if let Some(complete) = &mut self.upload_complete {
            if complete.draw(ctx) {
                self.upload_complete = None;
                self.refresh_files();
            }
        }

        if self.show_about {
            crate::ui::draw_about_window(ctx, &mut self.show_about, &mut self.about_icon_texture);
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
