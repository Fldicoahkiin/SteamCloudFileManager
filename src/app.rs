use crate::steam_worker::SteamWorkerManager;
use crate::vdf_parser::VdfParser;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct SteamCloudApp {
    // 核心服务
    steam_manager: Arc<Mutex<SteamWorkerManager>>,
    update_manager: crate::update::UpdateManager,

    // 业务逻辑处理器
    handlers: crate::app_handlers::AppHandlers,

    // 子状态
    connection: crate::app_state::ConnectionState,
    file_list: crate::app_state::FileListState,
    game_library: crate::app_state::GameLibraryState,
    dialogs: crate::app_state::DialogState,
    misc: crate::app_state::MiscState,

    // 异步处理器
    async_handlers: crate::async_handlers::AsyncHandlers,
}

impl Default for SteamCloudApp {
    fn default() -> Self {
        let steam_manager = Arc::new(Mutex::new(SteamWorkerManager::new()));
        let vdf_parser = VdfParser::new().ok();
        let handlers =
            crate::app_handlers::AppHandlers::new(steam_manager.clone(), vdf_parser.clone());

        Self {
            steam_manager,
            update_manager: crate::update::UpdateManager::new(),
            handlers,
            connection: Default::default(),
            file_list: Default::default(),
            game_library: Default::default(),
            dialogs: Default::default(),
            misc: Default::default(),
            async_handlers: Default::default(),
        }
    }
}

impl SteamCloudApp {
    fn show_error(&mut self, message: &str) {
        self.dialogs.show_error(message);
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::ui::setup_fonts(&cc.egui_ctx);

        let mut app = Self::default();
        app.scan_cloud_games();
        app
    }

    fn connect_to_steam(&mut self) {
        if self.connection.app_id_input.trim().is_empty() {
            self.show_error(self.misc.i18n.error_enter_app_id());
            return;
        }

        if self.connection.is_connecting || self.async_handlers.connect_rx.is_some() {
            return;
        }

        let app_id = match self.connection.app_id_input.trim().parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                self.show_error(self.misc.i18n.error_invalid_app_id());
                return;
            }
        };

        self.handlers.connect_to_steam(
            &mut self.connection,
            &mut self.misc,
            &mut self.async_handlers,
            app_id,
        );
    }

    fn disconnect_from_steam(&mut self) {
        self.handlers.disconnect_from_steam(
            &mut self.connection,
            &mut self.file_list,
            &mut self.misc,
        );
    }

    fn refresh_files(&mut self) {
        let _ = self.handlers.refresh_files(
            &self.connection,
            &mut self.file_list,
            &mut self.async_handlers,
        );
    }

    fn open_cloud_url(&mut self) {
        self.handlers.open_cloud_url(&self.connection.app_id_input);
    }

    fn download(&mut self) {
        if let Some((tasks, _base_dir)) =
            self.handlers
                .prepare_download(&self.file_list, &mut self.dialogs, &self.misc.i18n)
        {
            self.handlers
                .start_download(tasks, &mut self.dialogs, &mut self.async_handlers);
        }
    }

    fn upload(&mut self) {
        self.handlers
            .upload_files(&self.connection, &mut self.dialogs, &self.misc.i18n);
    }

    fn upload_start(&mut self, queue: crate::file_manager::UploadQueue) {
        self.handlers
            .start_upload(queue, &mut self.dialogs, &mut self.async_handlers);
    }

    fn forget(&mut self) {
        if self
            .handlers
            .forget_files(&self.file_list, &mut self.misc, &mut self.dialogs)
        {
            self.refresh_files();
        }
    }

    fn sync_to_cloud(&mut self) {
        if self
            .handlers
            .sync_to_cloud(&self.file_list, &mut self.misc, &mut self.dialogs)
        {
            self.refresh_files();
        }
    }

    fn delete(&mut self) {
        if self
            .handlers
            .delete_files(&self.file_list, &mut self.misc, &mut self.dialogs)
        {
            self.refresh_files();
        }
    }

    fn compare_files(&mut self) {
        let app_id = self.connection.app_id_input.parse::<u32>().unwrap_or(0);
        self.handlers
            .compare_files(&mut self.file_list, &mut self.dialogs, app_id);
    }

    fn show_appinfo(&mut self, app_id: u32) {
        match crate::vdf_parser::VdfParser::new() {
            Ok(parser) => match parser.get_ufs_config(app_id) {
                Ok(config) => {
                    self.dialogs.appinfo_dialog =
                        Some(crate::ui::AppInfoDialog::new(app_id, config));
                }
                Err(e) => {
                    self.dialogs.show_error(&format!("无法获取 appinfo: {}", e));
                }
            },
            Err(e) => {
                self.dialogs
                    .show_error(&format!("VDF 解析器初始化失败: {}", e));
            }
        }
    }

    fn scan_cloud_games(&mut self) {
        self.handlers.scan_cloud_games(
            &mut self.game_library,
            &mut self.misc,
            &mut self.async_handlers,
            &mut self.dialogs,
        );
    }

    fn handle_file_drop(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        // 检测文件拖拽悬停
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("file_drop_overlay"),
            ));

            let screen_rect = ctx.content_rect();
            painter.rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 150),
            );

            painter.text(
                screen_rect.center(),
                egui::Align2::CENTER_CENTER,
                self.misc.i18n.drop_files_to_upload(),
                egui::FontId::proportional(30.0),
                egui::Color32::WHITE,
            );
        }

        // 处理文件拖放
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            if !self.connection.is_connected {
                self.show_error(self.misc.i18n.error_not_connected());
                return;
            }

            let mut queue = crate::file_manager::UploadQueue::new();
            let mut added_count = 0;

            for file in dropped_files {
                if let Some(path) = &file.path {
                    if path.is_file() {
                        if let Err(e) = queue.add_file(path.to_path_buf()) {
                            tracing::warn!("添加文件失败 {}: {}", path.display(), e);
                        } else {
                            added_count += 1;
                        }
                    } else if path.is_dir() {
                        if let Err(e) = queue.add_folder(path) {
                            tracing::warn!("添加文件夹失败 {}: {}", path.display(), e);
                        } else {
                            added_count += 1;
                        }
                    }
                }
            }

            if added_count > 0 {
                // 显示预览对话框，而不是直接上传
                self.dialogs.upload_preview = Some(crate::ui::UploadPreviewDialog::new(queue));
            }
        }
    }

    fn poll_async_results(&mut self) {
        // 连接结果
        if let Some(result) = self.async_handlers.poll_connect() {
            if self.handlers.handle_connect_result(
                result,
                &mut self.connection,
                &mut self.misc,
                &mut self.dialogs,
            ) {
                self.refresh_files();
            }
        }

        // 文件加载结果
        if let Some(result) = self.async_handlers.poll_loader() {
            self.handlers.handle_loader_result(
                result,
                &mut self.connection,
                &mut self.file_list,
                &mut self.misc,
                &mut self.dialogs,
            );
        }

        // 游戏扫描结果
        if let Some(result) = self.async_handlers.poll_scan_games() {
            self.handlers.handle_scan_games_result(
                result,
                &mut self.game_library,
                &mut self.misc,
                &mut self.dialogs,
            );
        }

        // 上传进度
        if let Some(progress_data) = self.async_handlers.poll_upload_progress() {
            self.handlers
                .handle_upload_progress(progress_data, &mut self.dialogs);
        }

        // 上传结果
        if let Some(result) = self.async_handlers.poll_upload_result() {
            self.handlers
                .handle_upload_result(result, &mut self.dialogs, &self.misc.i18n);
        }

        // Hash 检测结果
        self.handlers
            .poll_hash_results(&mut self.file_list, &mut self.dialogs);

        // 更新下载进度
        self.update_manager.poll_progress();

        // 更新下载结果
        if let Some(result) = self.async_handlers.poll_update_download() {
            self.handlers
                .handle_update_download_result(result, &mut self.update_manager);
        }

        // 备份进度
        if let Some(ref rx) = self.async_handlers.backup_progress_rx {
            while let Ok(progress) = rx.try_recv() {
                if let Some(ref mut dialog) = self.dialogs.backup_progress {
                    dialog.progress = progress;
                }
            }
        }

        // 备份结果
        if let Some(ref rx) = self.async_handlers.backup_rx {
            if let Ok(result) = rx.try_recv() {
                if let Some(ref mut dialog) = self.dialogs.backup_progress {
                    dialog.set_result(result);
                }
                self.async_handlers.backup_rx = None;
                self.async_handlers.backup_progress_rx = None;
            }
        }

        // 下载进度
        if let Some(ref rx) = self.async_handlers.download_progress_rx {
            while let Ok(progress) = rx.try_recv() {
                if let Some(ref mut dialog) = self.dialogs.download_progress {
                    dialog.progress = progress;
                }
            }
        }

        // 下载结果
        if let Some(ref rx) = self.async_handlers.download_rx {
            if let Ok(result) = rx.try_recv() {
                if let Some(ref mut dialog) = self.dialogs.download_progress {
                    dialog.set_result(result);
                }
                self.async_handlers.download_rx = None;
                self.async_handlers.download_progress_rx = None;
            }
        }

        // Steam 重启状态
        if let Some(status) = self.async_handlers.poll_restart() {
            self.handlers
                .handle_restart_status(status, &mut self.dialogs, &self.misc.i18n);
        }
    }

    fn check_steam_connection(&mut self) {
        if self.connection.is_connected {
            if let Ok(manager) = self.steam_manager.try_lock() {
                manager.run_callbacks();
            }

            // 检查超时（30秒）
            if !self.connection.remote_ready && self.file_list.is_refreshing {
                if let Some(since) = self.connection.since_connected {
                    if since.elapsed() >= Duration::from_secs(30) {
                        tracing::warn!("Steam API 加载超时，停止等待");
                        self.file_list.is_refreshing = false;
                        self.connection.remote_ready = true;
                        self.async_handlers.loader_rx = None;
                        self.misc.status_message = "加载超时，请重试".to_string();
                    }
                }
            }
        }
    }
}

impl eframe::App for SteamCloudApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 动态更新窗口标题
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            self.misc.i18n.app_title().to_string(),
        ));

        // Steam 连接状态检查
        self.check_steam_connection();

        // 轮询所有异步结果
        self.poll_async_results();

        // 渲染顶部面板
        let top_event = egui::TopBottomPanel::top("top_panel")
            .show(ctx, |ui| {
                crate::ui::render_top_panel(
                    ui,
                    &mut self.dialogs,
                    &mut self.connection,
                    &mut self.game_library,
                    &self.file_list,
                    &mut self.async_handlers,
                    &mut self.misc,
                )
            })
            .inner;

        // 处理顶部面板事件
        match top_event {
            crate::ui::TopPanelEvent::ScanGames => self.scan_cloud_games(),
            crate::ui::TopPanelEvent::Connect => self.connect_to_steam(),
            crate::ui::TopPanelEvent::Disconnect => self.disconnect_from_steam(),
            crate::ui::TopPanelEvent::Refresh => self.open_cloud_url(),
            crate::ui::TopPanelEvent::Restart => {
                self.handlers
                    .start_restart_steam(ctx, &mut self.async_handlers);
            }
            crate::ui::TopPanelEvent::None => {}
        }

        // 渲染底部面板
        let bottom_event = egui::TopBottomPanel::bottom("bottom_panel")
            .show(ctx, |ui| {
                crate::ui::render_bottom_panel(
                    ui,
                    &self.connection,
                    &mut self.file_list,
                    &self.misc,
                    &self.game_library,
                    &self.steam_manager,
                )
            })
            .inner;

        // 处理底部面板事件
        match bottom_event {
            crate::ui::BottomPanelEvent::SelectAll => {
                self.file_list.selected_files =
                    crate::ui::select_all_files(self.file_list.files.len());
            }
            crate::ui::BottomPanelEvent::InvertSelection => {
                self.file_list.selected_files = crate::ui::invert_file_selection(
                    &self.file_list.selected_files,
                    self.file_list.files.len(),
                );
            }
            crate::ui::BottomPanelEvent::ClearSelection => {
                self.file_list.selected_files = crate::ui::clear_file_selection();
            }
            crate::ui::BottomPanelEvent::Download => self.download(),
            crate::ui::BottomPanelEvent::Upload => self.upload(),
            crate::ui::BottomPanelEvent::Delete => self.delete(),
            crate::ui::BottomPanelEvent::Forget => self.forget(),
            crate::ui::BottomPanelEvent::SyncToCloud => self.sync_to_cloud(),
            crate::ui::BottomPanelEvent::CompareFiles => self.compare_files(),
            crate::ui::BottomPanelEvent::ToggleCloud => {
                if let Ok(mut manager) = self.steam_manager.lock() {
                    if let Ok(enabled) = manager.is_cloud_enabled_for_app() {
                        let _ = manager.set_cloud_enabled_for_app(!enabled);
                    }
                }
            }
            crate::ui::BottomPanelEvent::ShowAppInfo(app_id) => {
                self.show_appinfo(app_id);
            }
            crate::ui::BottomPanelEvent::None => {}
        }

        // 渲染中心面板
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.connection.is_connected && self.connection.remote_ready {
                self.handle_file_drop(ctx, ui);
            }

            crate::ui::render_center_panel(ui, &self.connection, &mut self.file_list, &self.misc);
        });

        if self.dialogs.show_error
            && crate::ui::draw_error_window(
                ctx,
                &mut self.dialogs.show_error,
                &self.dialogs.error_message,
                &self.misc.i18n,
            )
        {
            self.dialogs.show_error = false;
        }

        // AppInfo 对话框
        if let Some(ref dialog) = self.dialogs.appinfo_dialog.clone() {
            if !crate::ui::draw_appinfo_dialog(ctx, dialog, &self.misc.i18n) {
                self.dialogs.appinfo_dialog = None;
            }
        }

        if self.game_library.show_game_selector {
            let (selected_app_id, refresh_clicked) = crate::ui::draw_game_selector_window(
                ctx,
                &mut self.game_library.show_game_selector,
                &self.game_library.cloud_games,
                self.game_library.is_scanning_games,
                self.game_library.vdf_count,
                self.game_library.cdp_count,
                &self.misc.i18n,
            );
            if let Some(app_id) = selected_app_id {
                self.connection.app_id_input = app_id.to_string();
                self.game_library.show_game_selector = false;
                self.connect_to_steam();
            }
            if refresh_clicked {
                tracing::info!("用户点击刷新游戏库");
                self.scan_cloud_games();
            }
        }

        // 绘制引导对话框
        let mut close_dialog = false;
        if let Some(dialog) = &mut self.dialogs.guide_dialog {
            let action = dialog.draw(ctx, &self.misc.i18n);
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
            self.dialogs.guide_dialog = None;
        }

        // 用户切换对话框
        if self.game_library.show_user_selector {
            if self.game_library.all_users.is_empty() {
                if let Ok(parser) = VdfParser::new() {
                    self.game_library.all_users = crate::user_manager::get_all_users_info(
                        parser.get_steam_path(),
                        parser.get_user_id(),
                    )
                    .unwrap_or_default();
                }
            }

            let selected_user_id = crate::ui::draw_user_selector_window(
                ctx,
                &mut self.game_library.show_user_selector,
                &self.game_library.all_users,
                &self.misc.i18n,
            );
            if let Some(user_id) = selected_user_id {
                if let Ok(parser) = VdfParser::new() {
                    let steam_path = parser.get_steam_path().clone();
                    let new_parser = VdfParser::with_user_id(steam_path, user_id);
                    self.handlers = crate::app_handlers::AppHandlers::new(
                        self.steam_manager.clone(),
                        Some(new_parser),
                    );
                    self.game_library.cloud_games.clear();
                    self.misc.status_message = self.misc.i18n.user_switched().to_string();
                    self.scan_cloud_games();
                }
                self.game_library.show_user_selector = false;
            }
        }

        // 上传预览对话框
        if let Some(preview) = &mut self.dialogs.upload_preview {
            match preview.draw(ctx, &self.misc.i18n) {
                crate::ui::UploadAction::Confirm => {
                    // 开始上传
                    if let Some(preview) = self.dialogs.upload_preview.take() {
                        self.upload_start(preview.queue);
                    }
                }
                crate::ui::UploadAction::Cancel => {
                    self.dialogs.upload_preview = None;
                }
                crate::ui::UploadAction::None => {}
            }
        }

        // 上传进度对话框
        if let Some(progress) = &mut self.dialogs.upload_progress {
            progress.draw(ctx, &self.misc.i18n);
            if !progress.show {
                self.dialogs.upload_progress = None;
            }
        }

        // 上传完成对话框
        if let Some(complete) = &mut self.dialogs.upload_complete {
            if complete.draw(ctx, &self.misc.i18n) {
                self.dialogs.upload_complete = None;
                self.refresh_files();
            }
        }

        // 文件对比对话框（只读信息展示）
        if let crate::ui::ConflictDialogEvent::RetryHashCheck(filename) =
            crate::ui::draw_conflict_dialog(ctx, &mut self.dialogs.conflict_dialog, &self.misc.i18n)
        {
            let app_id = self.connection.app_id_input.parse::<u32>().unwrap_or(0);
            self.handlers.retry_hash_check(
                &filename,
                &mut self.file_list,
                &mut self.dialogs,
                app_id,
            );
        }

        let was_showing_settings = self.dialogs.show_settings;
        if self.dialogs.show_settings {
            if let Some(release) = crate::ui::draw_settings_window(
                ctx,
                &mut self.dialogs.show_settings,
                &mut self.dialogs.settings_state,
                &mut self.update_manager,
                &self.misc.i18n,
            ) {
                // 启动异步下载
                let rx = self.update_manager.start_download(&release);
                self.async_handlers.update_download_rx = Some(rx);
            }
        }

        // 关闭设置窗口时，如果是 NoUpdate 状态则重置为 Idle
        if was_showing_settings
            && !self.dialogs.show_settings
            && matches!(
                self.update_manager.status(),
                crate::update::UpdateStatus::NoUpdate
            )
        {
            self.update_manager.reset();
        }

        // 备份对话框
        if self.dialogs.show_backup && self.dialogs.backup_preview.is_none() {
            // 创建备份预览对话框
            let app_id = self.connection.app_id_input.parse::<u32>().unwrap_or(0);
            let game_name = self
                .game_library
                .cloud_games
                .iter()
                .find(|g| g.app_id == app_id)
                .and_then(|g| g.game_name.clone())
                .unwrap_or_else(|| format!("Game {}", app_id));

            self.dialogs.backup_preview = Some(crate::ui::BackupPreviewDialog::new(
                app_id,
                game_name,
                self.file_list.files.clone(),
            ));
            self.dialogs.show_backup = false;
        }

        // 绘制备份预览对话框
        if let Some(ref mut preview) = self.dialogs.backup_preview {
            let action = preview.draw(ctx, &self.misc.i18n);
            match action {
                crate::ui::BackupAction::StartBackup => {
                    // 开始备份
                    let app_id = preview.app_id;
                    let game_name = preview.game_name.clone();
                    let files = preview.files.clone();
                    self.dialogs.backup_preview = None;

                    // 启动备份任务
                    self.handlers.start_backup(
                        app_id,
                        game_name,
                        files,
                        &mut self.dialogs,
                        &mut self.async_handlers,
                    );
                }
                crate::ui::BackupAction::Cancel => {
                    self.dialogs.backup_preview = None;
                }
                crate::ui::BackupAction::OpenBackupDir => {
                    if let Ok(manager) = crate::backup::BackupManager::new() {
                        let _ = manager.open_backup_dir();
                    }
                }
                crate::ui::BackupAction::None => {}
            }
        }

        // 绘制备份进度对话框
        if let Some(ref mut progress_dialog) = self.dialogs.backup_progress {
            match progress_dialog.draw(ctx, &self.misc.i18n) {
                crate::ui::ProgressAction::Cancel => {
                    self.async_handlers.cancel_backup();
                }
                crate::ui::ProgressAction::Close => {
                    self.dialogs.backup_progress = None;
                    self.async_handlers.backup_progress_rx = None;
                    self.async_handlers.backup_cancel = None;
                }
                crate::ui::ProgressAction::None => {}
            }
        }

        // 绘制下载进度对话框
        if let Some(ref mut progress_dialog) = self.dialogs.download_progress {
            match progress_dialog.draw(ctx, &self.misc.i18n) {
                crate::ui::ProgressAction::Cancel => {
                    self.async_handlers.cancel_download();
                }
                crate::ui::ProgressAction::Close => {
                    self.dialogs.download_progress = None;
                    self.async_handlers.download_progress_rx = None;
                    self.async_handlers.download_cancel = None;
                }
                crate::ui::ProgressAction::None => {}
            }
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
