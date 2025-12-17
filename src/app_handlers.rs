use crate::app_state::{ConnectionState, DialogState, FileListState, GameLibraryState, MiscState};
use crate::async_handlers::AsyncHandlers;
use crate::steam_worker::SteamWorkerManager;
use crate::vdf_parser::VdfParser;
use anyhow::{anyhow, Result as AnyhowResult};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct AppHandlers {
    steam_manager: Arc<Mutex<SteamWorkerManager>>,
    vdf_parser: Option<VdfParser>,
}

impl AppHandlers {
    pub fn new(
        steam_manager: Arc<Mutex<SteamWorkerManager>>,
        vdf_parser: Option<VdfParser>,
    ) -> Self {
        Self {
            steam_manager,
            vdf_parser,
        }
    }

    pub fn connect_to_steam(
        &self,
        connection: &mut ConnectionState,
        misc: &mut MiscState,
        async_handlers: &mut AsyncHandlers,
        app_id: u32,
    ) {
        connection.reset();
        connection.is_connecting = true;
        misc.status_message = misc.i18n.connecting_to_steam(app_id);

        let rx = SteamWorkerManager::connect_async(self.steam_manager.clone(), app_id);
        async_handlers.connect_rx = Some(rx);
    }

    pub fn disconnect_from_steam(
        &self,
        connection: &mut ConnectionState,
        file_list: &mut FileListState,
        misc: &mut MiscState,
    ) {
        if let Ok(mut manager) = self.steam_manager.lock() {
            manager.disconnect();
        }

        connection.reset();
        file_list.clear();
        misc.quota_info = None;
        misc.status_message = "已断开连接".to_string();
    }

    pub fn refresh_files(
        &self,
        connection: &ConnectionState,
        file_list: &mut FileListState,
        async_handlers: &mut AsyncHandlers,
    ) -> AnyhowResult<()> {
        if !connection.is_connected {
            return Err(anyhow!("Steam 未连接"));
        }

        if async_handlers.loader_rx.is_some() {
            return Ok(());
        }

        file_list.is_refreshing = true;
        file_list.files.clear();

        let steam_manager = self.steam_manager.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        async_handlers.loader_rx = Some(rx);

        let app_id = connection.app_id_input.trim().parse::<u32>().unwrap_or(0);

        std::thread::spawn(move || {
            let file_service = crate::file_manager::FileService::with_steam_manager(steam_manager);

            let files = match file_service.get_cloud_files(app_id) {
                Ok(files) => {
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

        Ok(())
    }

    pub fn update_quota(&self, misc: &mut MiscState) {
        if let Ok(mut manager) = self.steam_manager.lock() {
            if let Ok((total, available)) = manager.get_quota() {
                misc.quota_info = Some((total, available));
            }
        }
    }

    pub fn open_cloud_url(&self, app_id_input: &str) {
        if let Ok(app_id) = app_id_input.parse::<u32>() {
            let steam_url = format!(
                "steam://openurl/https://store.steampowered.com/account/remotestorageapp/?appid={}",
                app_id
            );
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg(&steam_url).spawn();
            }
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "", &steam_url])
                    .spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&steam_url)
                    .spawn();
            }
        }
    }

    pub fn download_files(
        &self,
        file_list: &FileListState,
        misc: &mut MiscState,
        dialogs: &mut DialogState,
    ) {
        let file_ops = crate::file_manager::FileOperations::new(self.steam_manager.clone());
        match file_ops.download_by_indices(
            &file_list.files,
            &file_list.selected_files,
            &file_list.local_save_paths,
        ) {
            Ok(Some((success_count, failed_files))) => {
                if failed_files.is_empty() {
                    misc.status_message = misc.i18n.download_success(success_count);
                } else {
                    let error_msg = misc.i18n.download_partial(
                        success_count,
                        failed_files.len(),
                        &failed_files.join(", "),
                    );
                    dialogs.show_error(&error_msg);
                }
            }
            Ok(None) => {}
            Err(e) => {
                dialogs.show_error(&misc.i18n.download_failed(&e.to_string()));
            }
        }
    }

    pub fn upload_files(
        &self,
        connection: &ConnectionState,
        dialogs: &mut DialogState,
        i18n: &crate::i18n::I18n,
    ) {
        if !connection.is_connected {
            dialogs.show_error(i18n.error_not_connected());
            return;
        }

        // 直接打开空的上传准备对话框，用户在对话框中添加文件
        let queue = crate::file_manager::UploadQueue::new();
        dialogs.upload_preview = Some(crate::ui::UploadPreviewDialog::new(queue));
    }

    pub fn start_upload(
        &self,
        queue: crate::file_manager::UploadQueue,
        dialogs: &mut DialogState,
        async_handlers: &mut AsyncHandlers,
    ) {
        let total_files = queue.total_files();
        dialogs.upload_progress = Some(crate::ui::UploadProgressDialog::new(total_files));

        let steam_manager = self.steam_manager.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let (progress_tx, progress_rx) = std::sync::mpsc::channel();
        async_handlers.upload_rx = Some(rx);
        async_handlers.upload_progress_rx = Some(progress_rx);

        std::thread::spawn(move || {
            let mut queue = queue;
            let executor = crate::file_manager::UploadExecutor::new(steam_manager)
                .with_progress_callback(move |current, total, filename| {
                    let _ = progress_tx.send((current, total, filename.to_string()));
                });

            match executor.execute(&mut queue) {
                Ok(result) => {
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

    pub fn forget_files(
        &self,
        file_list: &FileListState,
        misc: &mut MiscState,
        dialogs: &mut DialogState,
    ) -> bool {
        use crate::file_manager::FileOperationResult;

        let file_ops = crate::file_manager::FileOperations::new(self.steam_manager.clone());
        let result = file_ops.forget_by_indices(&file_list.files, &file_list.selected_files);

        match result {
            FileOperationResult::SuccessWithRefresh(msg) => {
                misc.status_message = msg;
                true
            }
            FileOperationResult::Error(msg) => {
                dialogs.show_error(&msg);
                false
            }
        }
    }

    pub fn delete_files(
        &self,
        file_list: &FileListState,
        misc: &mut MiscState,
        dialogs: &mut DialogState,
    ) -> bool {
        use crate::file_manager::FileOperationResult;

        let file_ops = crate::file_manager::FileOperations::new(self.steam_manager.clone());
        let result = file_ops.delete_by_indices(&file_list.files, &file_list.selected_files);

        match result {
            FileOperationResult::SuccessWithRefresh(msg) => {
                misc.status_message = msg;
                true
            }
            FileOperationResult::Error(msg) => {
                dialogs.show_error(&msg);
                false
            }
        }
    }

    pub fn scan_cloud_games(
        &mut self,
        game_library: &mut GameLibraryState,
        misc: &mut MiscState,
        async_handlers: &mut AsyncHandlers,
        dialogs: &mut DialogState,
    ) {
        let parser_data = self
            .ensure_vdf_parser()
            .map(|p| (p.get_steam_path().clone(), p.get_user_id().to_string()));

        if let Some((steam_path, user_id)) = parser_data {
            let steam_url = "steam://openurl/https://store.steampowered.com/account/remotestorage";
            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open").arg(steam_url).spawn();
            #[cfg(target_os = "windows")]
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", "", steam_url])
                .spawn();
            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open")
                .arg(steam_url)
                .spawn();

            game_library.is_scanning_games = true;
            misc.status_message = misc.i18n.scanning_game_library().to_string();
            let (tx, rx) = std::sync::mpsc::channel();
            async_handlers.scan_games_rx = Some(rx);

            std::thread::spawn(move || {
                let result = crate::game_scanner::fetch_and_merge_games(steam_path, user_id)
                    .map_err(|e| e.to_string());
                let _ = tx.send(result);
            });
        } else {
            dialogs.show_error(misc.i18n.vdf_parser_not_initialized());
        }
    }

    fn ensure_vdf_parser(&mut self) -> Option<&VdfParser> {
        if self.vdf_parser.is_none() {
            self.vdf_parser = VdfParser::new().ok();
        }
        self.vdf_parser.as_ref()
    }

    pub fn handle_connect_result(
        &self,
        result: Result<u32, String>,
        connection: &mut ConnectionState,
        misc: &mut MiscState,
        dialogs: &mut DialogState,
    ) -> bool {
        match result {
            Ok(app_id) => {
                connection.is_connecting = false;
                connection.is_connected = true;
                misc.status_message = misc.i18n.loading_files_for_app(app_id);
                connection.since_connected = Some(Instant::now());

                // 打开对应 app_id 的云存储页面
                let steam_url = format!(
                    "steam://openurl/https://store.steampowered.com/account/remotestorageapp/?appid={}",
                    app_id
                );
                #[cfg(target_os = "macos")]
                let _ = std::process::Command::new("open").arg(&steam_url).spawn();
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "", &steam_url])
                    .spawn();
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open")
                    .arg(&steam_url)
                    .spawn();

                true
            }
            Err(err) => {
                connection.is_connecting = false;
                dialogs.show_error(&misc.i18n.connect_steam_failed(&err));
                false
            }
        }
    }

    pub fn handle_loader_result(
        &mut self,
        result: Result<Vec<crate::steam_api::CloudFile>, String>,
        connection: &mut ConnectionState,
        file_list: &mut FileListState,
        misc: &mut MiscState,
        dialogs: &mut DialogState,
    ) {
        match result {
            Ok(files) => {
                let count = files.len();
                file_list.files = files;
                file_list.selected_files.clear();
                self.update_quota(misc);

                if let Ok(app_id) = connection.app_id_input.parse::<u32>() {
                    let parser_data = self
                        .ensure_vdf_parser()
                        .map(|p| (p.get_steam_path().clone(), p.get_user_id().to_string()));

                    if let Some((steam_path, user_id)) = parser_data {
                        file_list.local_save_paths = crate::path_resolver::collect_local_save_paths(
                            &file_list.files,
                            &steam_path,
                            &user_id,
                            app_id,
                        );
                    } else {
                        file_list.local_save_paths.clear();
                    }
                } else {
                    file_list.local_save_paths.clear();
                }

                file_list.file_tree = Some(crate::file_tree::FileTree::new(&file_list.files));
                misc.status_message = misc.i18n.status_files_loaded(count);
                file_list.is_refreshing = false;
                connection.remote_ready = true;
            }
            Err(err) => {
                dialogs.show_error(&misc.i18n.refresh_files_failed(&err));
                file_list.is_refreshing = false;
            }
        }
    }

    pub fn handle_scan_games_result(
        &self,
        result: Result<crate::game_scanner::ScanResult, String>,
        game_library: &mut GameLibraryState,
        misc: &mut MiscState,
        dialogs: &mut DialogState,
    ) {
        match result {
            Ok(result) => {
                game_library.cloud_games = result.games;
                game_library.vdf_count = result.vdf_count;
                game_library.cdp_count = result.cdp_count;

                // 状态消息将在 UI 渲染时动态生成，以支持语言切换
                misc.status_message = String::new();

                if result.cdp_count == 0 && crate::cdp_client::CdpClient::is_cdp_running() {
                    dialogs.show_error(misc.i18n.cdp_no_data_error());
                }

                game_library.is_scanning_games = false;
            }
            Err(err) => {
                dialogs.show_error(&misc.i18n.scan_games_failed(&err));
                game_library.is_scanning_games = false;
            }
        }
    }

    pub fn handle_upload_progress(
        &self,
        progress_data: (usize, usize, String),
        dialogs: &mut DialogState,
    ) {
        let (current, total, filename) = progress_data;
        if let Some(progress) = &mut dialogs.upload_progress {
            progress.current_index = current;
            progress.total_files = total;
            progress.current_file = filename.clone();
            progress.progress = current as f32 / total as f32;

            if current > progress.completed_files.len() {
                progress.completed_files.push(filename);
            }
        }
    }

    pub fn handle_upload_result(
        &self,
        result: Result<String, String>,
        dialogs: &mut DialogState,
        i18n: &crate::i18n::I18n,
    ) {
        match result {
            Ok(msg) => {
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

                    dialogs.upload_progress = None;
                    dialogs.upload_complete = Some(crate::ui::UploadCompleteDialog::new(
                        success_count,
                        failed_count,
                        total_size,
                        elapsed_secs,
                        failed_files,
                    ));
                }
            }
            Err(err) => {
                dialogs.upload_progress = None;
                dialogs.show_error(&i18n.upload_failed(&err));
            }
        }
    }

    pub fn handle_restart_status(
        &self,
        status: crate::steam_process::RestartStatus,
        dialogs: &mut DialogState,
        i18n: &crate::i18n::I18n,
    ) {
        use crate::steam_process::RestartStatus;
        match status {
            RestartStatus::Closing => {
                if let Some(dialog) = &mut dialogs.guide_dialog {
                    dialog.update_status(i18n.closing_steam().to_string(), false, false);
                }
            }
            RestartStatus::Starting => {
                if let Some(dialog) = &mut dialogs.guide_dialog {
                    dialog.update_status(i18n.starting_steam().to_string(), false, false);
                }
            }
            RestartStatus::Success => {
                if let Some(dialog) = &mut dialogs.guide_dialog {
                    dialog.update_status(i18n.steam_restart_success().to_string(), true, false);
                }
            }
            RestartStatus::Error(_msg) => {
                #[cfg(target_os = "macos")]
                {
                    dialogs.guide_dialog = Some(crate::ui::create_macos_manual_guide(i18n));
                }
                #[cfg(target_os = "windows")]
                {
                    dialogs.guide_dialog = Some(crate::ui::create_windows_manual_guide(i18n));
                }
                #[cfg(target_os = "linux")]
                {
                    dialogs.guide_dialog = Some(crate::ui::create_linux_manual_guide(i18n));
                }
            }
        }
    }
}
