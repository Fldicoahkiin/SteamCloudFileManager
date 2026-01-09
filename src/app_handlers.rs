use crate::app_state::{ConnectionState, DialogState, FileListState, GameLibraryState, MiscState};
use crate::async_handlers::AsyncHandlers;
use crate::steam_worker::SteamWorkerManager;
use crate::vdf_parser::VdfParser;
use anyhow::{anyhow, Result as AnyhowResult};
use chrono::TimeZone;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct AppHandlers {
    pub steam_manager: Arc<Mutex<SteamWorkerManager>>,
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
        file_list.hash_checked_app_id = None;
        file_list.hash_checker.cancel();

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

    // 从 appinfo.vdf 获取配额信息
    pub fn update_quota(&self, misc: &mut MiscState, app_id: u32) {
        if app_id == 0 {
            misc.quota_info = None;
            return;
        }

        // 仅从 appinfo.vdf 获取配额
        let quota_result =
            crate::vdf_parser::VdfParser::new().and_then(|parser| parser.get_ufs_config(app_id));

        match quota_result {
            Ok(config) if config.quota > 0 => {
                let total = config.quota;
                // 计算已用空间
                let used = if let Ok(mut manager) = self.steam_manager.lock() {
                    manager.calculate_used_space().unwrap_or(0)
                } else {
                    0
                };
                let available = total.saturating_sub(used);
                misc.quota_info = Some((total, available));
            }
            _ => {
                // appinfo.vdf 无配额数据时，不显示配额信息
                misc.quota_info = None;
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
                use std::os::windows::process::CommandExt;
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "", &steam_url])
                    .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
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

    // 准备异步下载（返回下载任务，由调用方启动异步下载）
    pub fn prepare_download(
        &self,
        file_list: &FileListState,
        dialogs: &mut DialogState,
        i18n: &crate::i18n::I18n,
    ) -> Option<(Vec<crate::downloader::DownloadTask>, std::path::PathBuf)> {
        if file_list.selected_files.is_empty() {
            dialogs.show_error(i18n.error_no_files_selected());
            return None;
        }

        // 选择下载目录
        let base_dir = crate::file_manager::FileOperations::pick_download_folder()?;

        // 准备下载任务
        let tasks = crate::file_manager::FileOperations::prepare_download_tasks(
            &file_list.files,
            &file_list.selected_files,
            &base_dir,
            &file_list.local_save_paths,
        );

        if tasks.is_empty() {
            dialogs.show_error(i18n.error_no_files_selected());
            return None;
        }

        Some((tasks, base_dir))
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

    pub fn sync_to_cloud(
        &self,
        file_list: &FileListState,
        misc: &mut MiscState,
        dialogs: &mut DialogState,
    ) -> bool {
        use crate::file_manager::FileOperationResult;

        let file_ops = crate::file_manager::FileOperations::new(self.steam_manager.clone());
        let result = file_ops.sync_to_cloud_by_indices(
            &file_list.files,
            &file_list.selected_files,
            &file_list.local_save_paths,
        );

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
        let result = file_ops.delete_by_indices(
            &file_list.files,
            &file_list.selected_files,
            &file_list.local_save_paths,
        );

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
            {
                use std::os::windows::process::CommandExt;
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "", steam_url])
                    .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
                    .spawn();
            }
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
                {
                    use std::os::windows::process::CommandExt;
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "", &steam_url])
                        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
                        .spawn();
                }
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

                let app_id = connection.app_id_input.parse::<u32>().unwrap_or(0);
                self.update_quota(misc, app_id);

                if app_id > 0 {
                    let parser_data = self
                        .ensure_vdf_parser()
                        .map(|p| (p.get_steam_path().clone(), p.get_user_id().to_string()));

                    if let Some((steam_path, user_id)) = parser_data {
                        // 从 appinfo.vdf 获取 savefiles 配置
                        let savefiles = self
                            .ensure_vdf_parser()
                            .and_then(|p| p.get_ufs_config(app_id).ok())
                            .map(|c| c.savefiles)
                            .unwrap_or_default();

                        // 基于 appinfo.vdf 收集本地存档路径（默认包含 root=0）
                        file_list.local_save_paths =
                            crate::path_resolver::collect_local_save_paths_from_ufs(
                                &savefiles,
                                &steam_path,
                                &user_id,
                                app_id,
                            );

                        // 如果没有 savefiles 配置，默认扫描 root=0 (SteamRemote)
                        let scan_savefiles = if savefiles.is_empty() {
                            tracing::debug!("appinfo.vdf 无 savefiles 配置，默认扫描 SteamRemote");
                            vec![crate::path_resolver::SaveFileConfig {
                                root: "0".to_string(),
                                root_type: Some(crate::path_resolver::RootType::SteamRemote),
                                path: String::new(),
                                pattern: "*".to_string(),
                                platforms: vec![],
                                recursive: true,
                            }]
                        } else {
                            savefiles
                        };

                        let cloud_filenames: std::collections::HashSet<_> =
                            file_list.files.iter().map(|f| f.name.as_str()).collect();

                        let scanned = crate::path_resolver::scan_local_files_from_ufs(
                            &scan_savefiles,
                            &steam_path,
                            &user_id,
                            app_id,
                        );

                        // 过滤出云端没有的文件，转换为 CloudFile 并添加到主列表
                        let local_only: Vec<_> = scanned
                            .into_iter()
                            .filter(|f| !cloud_filenames.contains(f.relative_path.as_str()))
                            .collect();

                        if !local_only.is_empty() {
                            tracing::info!("发现 {} 个本地独有文件 (云端无)", local_only.len());

                            // 转换为 CloudFile 并添加到文件列表
                            for local_file in local_only {
                                let timestamp = local_file
                                    .modified
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_secs() as i64)
                                    .unwrap_or(0);

                                let cloud_file = crate::steam_api::CloudFile {
                                    name: local_file.relative_path,
                                    size: local_file.size,
                                    timestamp: chrono::Local
                                        .timestamp_opt(timestamp, 0)
                                        .single()
                                        .unwrap_or_default(),
                                    is_persisted: false,
                                    exists: true, // 本地存在
                                    root: local_file.root_id,
                                    root_description: crate::path_resolver::get_root_description(
                                        local_file.root_id,
                                    ),
                                };
                                file_list.files.push(cloud_file);
                            }
                        }
                    } else {
                        file_list.local_save_paths.clear();
                    }
                } else {
                    file_list.local_save_paths.clear();
                }

                file_list.file_tree = Some(crate::file_tree::FileTree::new(&file_list.files));
                let comparisons = file_list.update_sync_status(); // 更新同步状态

                // 自动启动 Hash 检测
                if app_id > 0 && !file_list.files.is_empty() {
                    dialogs.conflict_dialog.set_comparisons(comparisons.clone());
                    file_list.hash_checker.start(app_id, &comparisons);
                    file_list.hash_checked_app_id = None; // 正在检测中，尚未完成
                    tracing::info!("已启动异步 Hash 检测");
                }

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

    // 执行文件对比检测
    pub fn compare_files(
        &self,
        file_list: &mut FileListState,
        dialogs: &mut DialogState,
        app_id: u32,
    ) {
        // 如果已经完成过 Hash 检测且是同一个 app_id，直接打开对话框显示缓存结果
        if file_list.hash_checked_app_id == Some(app_id) {
            tracing::debug!("使用缓存的 Hash 检测结果 (app_id={})", app_id);
            dialogs.conflict_dialog.show = true;
            return;
        }

        // 如果正在检测中，只显示对话框，不启动新检测（避免取消正在进行的自动检测）
        if file_list.hash_checker.is_running() && file_list.hash_checker.get_app_id() == app_id {
            tracing::info!("Hash 检测正在进行中，显示对话框 (app_id={})", app_id);
            dialogs.conflict_dialog.show = true;
            return;
        }

        // 没有检测过或检测被取消，启动新检测
        let comparisons =
            crate::conflict::detect_all(&file_list.files, &file_list.local_save_paths);
        dialogs.conflict_dialog.set_comparisons(comparisons.clone());
        dialogs.conflict_dialog.show = true;

        file_list.hash_checker.start(app_id, &comparisons);
        tracing::info!("已启动异步 Hash 检测 (app_id={})", app_id);
    }

    // 重新检测单个文件的 hash
    pub fn retry_hash_check(
        &self,
        filename: &str,
        file_list: &mut FileListState,
        dialogs: &mut DialogState,
        app_id: u32,
    ) {
        // 找到并克隆对应的 comparison
        let comparison = dialogs
            .conflict_dialog
            .comparisons
            .iter()
            .find(|c| c.filename == filename)
            .cloned();

        if let Some(comparison) = comparison {
            // 更新状态为 Checking
            if let Some(c) = dialogs
                .conflict_dialog
                .comparisons
                .iter_mut()
                .find(|c| c.filename == filename)
            {
                c.hash_status = crate::conflict::HashStatus::Checking;
            }

            file_list.hash_checker.start(app_id, &[comparison]);
            tracing::info!("重新检测文件 Hash: {}", filename);
        }
    }

    // 轮询 Hash 检测结果
    pub fn poll_hash_results(&self, file_list: &mut FileListState, dialogs: &mut DialogState) {
        for result in file_list.hash_checker.poll() {
            if let Some(ref err) = result.error {
                tracing::error!("Hash 检测失败: 文件={} 错误={}", result.filename, err);
            }
            let (hash_status, _) = result.process();

            // 更新 comparison_map
            if let Some(info) = file_list.comparison_map.get_mut(&result.filename) {
                info.hash_status = hash_status;
                info.diff_flags.hash_diff = hash_status == crate::conflict::HashStatus::Mismatch;

                // Hash 一致 = 已同步
                // Hash 不一致 = 保持原状态（LocalNewer/CloudNewer），或设为 Conflict
                let new_status = match hash_status {
                    crate::conflict::HashStatus::Match => crate::conflict::SyncStatus::Synced,
                    crate::conflict::HashStatus::Mismatch => {
                        if info.status == crate::conflict::SyncStatus::Unknown
                            || info.status == crate::conflict::SyncStatus::Synced
                        {
                            crate::conflict::SyncStatus::Conflict
                        } else {
                            info.status // 保持 LocalNewer/CloudNewer
                        }
                    }
                    _ => info.status,
                };
                info.status = new_status;
                file_list
                    .sync_status_map
                    .insert(result.filename.clone(), new_status);
            }

            // 更新对比窗口
            dialogs.conflict_dialog.update_hash_result(
                &result.filename,
                result.local_hash,
                result.cloud_hash,
                result.error.is_some(),
            );
        }

        // 检测完成后标记 app_id，用于缓存结果避免重复检测
        if file_list.hash_checked_app_id.is_none() && file_list.hash_checker.is_completed() {
            let app_id = file_list.hash_checker.get_app_id();
            if app_id > 0 {
                file_list.hash_checked_app_id = Some(app_id);
                tracing::info!("Hash 检测完成，已缓存结果 (app_id={})", app_id);
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

    pub fn start_download(
        &self,
        tasks: Vec<crate::downloader::DownloadTask>,
        dialogs: &mut DialogState,
        async_handlers: &mut AsyncHandlers,
    ) {
        let total_files = tasks.len();
        dialogs.download_progress = Some(crate::ui::DownloadProgressDialog::new(total_files));

        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let (progress_tx, progress_rx) = std::sync::mpsc::channel();
        let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let steam_manager = self.steam_manager.clone();

        async_handlers.download_rx = Some(result_rx);
        async_handlers.download_progress_rx = Some(progress_rx);
        async_handlers.download_cancel = Some(cancel_flag.clone());

        std::thread::spawn(move || {
            let downloader = crate::downloader::BatchDownloader::new(tasks)
                .with_cancel_flag(cancel_flag)
                .with_progress_sender(progress_tx)
                .with_steam_manager(steam_manager);

            let result = downloader.execute();
            let _ = result_tx.send(result);
        });
    }

    pub fn start_backup(
        &self,
        app_id: u32,
        game_name: String,
        files: Vec<crate::steam_api::CloudFile>,
        dialogs: &mut DialogState,
        async_handlers: &mut AsyncHandlers,
    ) {
        dialogs.backup_progress = Some(crate::ui::BackupProgressDialog::new(files.len()));

        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let (progress_tx, progress_rx) = std::sync::mpsc::channel();
        let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        async_handlers.backup_rx = Some(result_rx);
        async_handlers.backup_progress_rx = Some(progress_rx);
        async_handlers.backup_cancel = Some(cancel_flag.clone());

        std::thread::spawn(move || {
            let result = match crate::backup::BackupManager::new() {
                Ok(manager) => {
                    manager.create_backup(app_id, &game_name, &files, cancel_flag, |progress| {
                        let _ = progress_tx.send(progress.clone());
                    })
                }
                Err(e) => Err(e),
            };

            match result {
                Ok(backup_result) => {
                    let _ = result_tx.send(backup_result);
                }
                Err(e) => {
                    let _ = result_tx.send(crate::backup::BackupResult {
                        success: false,
                        backup_path: std::path::PathBuf::new(),
                        total_files: files.len(),
                        success_count: 0,
                        failed_files: vec![("backup".to_string(), e.to_string())],
                    });
                }
            }
        });
    }

    pub fn start_restart_steam(&self, ctx: &egui::Context, async_handlers: &mut AsyncHandlers) {
        let (tx, rx) = std::sync::mpsc::channel();
        async_handlers.restart_rx = Some(rx);
        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            crate::steam_process::restart_steam_with_status(tx, move || {
                ctx_clone.request_repaint();
            });
        });
    }

    pub fn handle_update_download_result(
        &self,
        result: Result<std::path::PathBuf, String>,
        update_manager: &mut crate::update::UpdateManager,
    ) {
        match result {
            Ok(download_path) => {
                tracing::info!("下载完成: {}", download_path.display());

                // 三平台统一使用自动安装
                if let Err(e) = update_manager.install_downloaded_update(&download_path) {
                    update_manager.set_error(format!("安装失败: {}\n\n请手动下载更新", e));
                }
            }
            Err(err) => {
                tracing::error!("下载失败: {}", err);
                update_manager.set_error(format!("下载失败: {}\n\n请手动下载更新", err));
            }
        }
    }
}
