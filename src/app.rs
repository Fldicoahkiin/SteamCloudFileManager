use crate::error::{AppError, AppResult};
use crate::game_scanner::CloudGameInfo;
use crate::steam_api::{CloudFile, SteamCloudManager};
use crate::vdf_parser::{UserInfo, VdfParser};
use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(PartialEq, Clone, Copy)]
enum SortColumn {
    Name,
    Size,
    Time,
}

#[derive(PartialEq, Clone, Copy, Default)]
enum SortOrder {
    Ascending,
    Descending,
    #[default]
    None,
}

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
    sort_column: Option<SortColumn>,
    sort_order: SortOrder,
    local_save_paths: Vec<(String, PathBuf)>,
    search_query: String,
    show_only_local: bool,
    show_only_cloud: bool,
    multi_select_mode: bool,
    cloud_games: Vec<CloudGameInfo>,
    show_game_selector: bool,
    is_scanning_games: bool,
    scan_games_rx: Option<Receiver<Result<Vec<CloudGameInfo>, String>>>,
    vdf_parser: Option<VdfParser>,
    all_users: Vec<UserInfo>,
    show_user_selector: bool,
    show_about: bool,
    show_debug_warning: bool,
    about_icon_texture: Option<egui::TextureHandle>,
}

impl SteamCloudApp {
    // 错误处理辅助方法
    fn handle_error(&mut self, error: AppError) {
        tracing::error!(error = ?error, "操作失败");
        self.show_error(&error.to_string());
    }

    // 获取或初始化 VdfParser
    fn ensure_vdf_parser(&mut self) -> Option<&VdfParser> {
        if self.vdf_parser.is_none() {
            self.vdf_parser = VdfParser::new().ok();
        }
        self.vdf_parser.as_ref()
    }

    // 通用的批量文件操作方法
    fn batch_file_operation<F>(
        &mut self,
        operation: F,
        _operation_name: &str,
    ) -> (usize, Vec<String>)
    where
        F: Fn(&str) -> anyhow::Result<bool>,
    {
        let filenames: Vec<String> = self
            .selected_files
            .iter()
            .map(|&index| &self.files[index].name)
            .cloned()
            .collect();

        let mut success_count = 0;
        let mut failed_files = Vec::new();

        for filename in &filenames {
            match operation(filename) {
                Ok(true) => success_count += 1,
                Ok(false) => failed_files.push(filename.to_string()),
                Err(e) => failed_files.push(format!("{} (错误: {})", filename, e)),
            }
        }

        (success_count, failed_files)
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

    fn get_selected_file_index(&self) -> AppResult<usize> {
        if self.selected_files.is_empty() {
            return Err(AppError::FileNotSelected);
        }
        Ok(self.selected_files[0])
    }

    // 解析配置文件（示例）
    #[allow(dead_code)]
    fn parse_config_value(&self, value: &str) -> AppResult<u32> {
        value
            .parse::<u32>()
            .map_err(|e| AppError::ParseError(format!("解析配置值失败: {}", e)))
    }

    fn draw_file_items_table(&mut self, body: egui_extras::TableBody) {
        let row_height = 20.0;
        let files: Vec<(usize, &CloudFile)> = self
            .files
            .iter()
            .enumerate()
            .filter(|(_, file)| {
                if self.show_only_local && file.exists {
                    return false;
                }
                if self.show_only_cloud && !file.exists {
                    return false;
                }
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    if !file.name.to_lowercase().contains(&query) {
                        return false;
                    }
                }
                true
            })
            .collect();

        body.rows(row_height, files.len(), |mut row| {
            let row_index = row.index();
            if let Some((index, file)) = files.get(row_index) {
                let index = *index;
                let is_selected = self.selected_files.contains(&index);

                row.col(|ui| {
                    // 文件夹列显示 root_description (e.g. WinAppDataLocal)
                    // 如果是 CDP 文件，root_description 格式为 CDP:<URL>|<FOLDER>
                    let display_folder = if file.root_description.starts_with("CDP:") {
                        file.root_description
                            .split('|')
                            .nth(1)
                            .unwrap_or("CDP File")
                    } else {
                        &file.root_description
                    };
                    ui.label(display_folder)
                        .on_hover_text(&file.root_description);
                });

                row.col(|ui| {
                    #[allow(deprecated)]
                    let response =
                        ui.add(egui::SelectableLabel::new(is_selected, &file.name).truncate());

                    if response.clicked() {
                        let modifiers = ui.ctx().input(|i| i.modifiers);
                        let ctrl = modifiers.ctrl || modifiers.command;
                        let shift = modifiers.shift;

                        if self.multi_select_mode || ctrl {
                            if is_selected {
                                self.selected_files.retain(|&x| x != index);
                            } else {
                                self.selected_files.push(index);
                            }
                        } else if shift {
                            if let Some(&last) = self.selected_files.last() {
                                let (min, max) = if last < index {
                                    (last, index)
                                } else {
                                    (index, last)
                                };
                                for i in min..=max {
                                    if !self.selected_files.contains(&i) {
                                        self.selected_files.push(i);
                                    }
                                }
                            } else {
                                self.selected_files.push(index);
                            }
                        } else {
                            self.selected_files.clear();
                            self.selected_files.push(index);
                        }
                    }
                });

                row.col(|ui| {
                    ui.label(crate::utils::format_size(file.size));
                });

                row.col(|ui| {
                    ui.label(file.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
                });

                row.col(|ui| {
                    if file.exists {
                        ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "✓");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "✗");
                    }
                });

                row.col(|ui| {
                    if file.is_persisted {
                        ui.colored_label(egui::Color32::from_rgb(0, 150, 255), "✓");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "✗");
                    }
                });
            }
        });
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();

        #[cfg(target_os = "windows")]
        {
            if let Ok(windir) = std::env::var("WINDIR") {
                let symbols_path = std::path::PathBuf::from(&windir)
                    .join("Fonts")
                    .join("seguisym.ttf");
                if let Ok(data) = std::fs::read(&symbols_path) {
                    fonts.font_data.insert(
                        "symbols".to_owned(),
                        egui::FontData::from_owned(data).into(),
                    );
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .push("symbols".to_owned());
                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("symbols".to_owned());
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let candidates = [
                "/System/Library/Fonts/Apple Symbols.ttf",
                "/System/Library/Fonts/Supplemental/Symbols.ttf",
            ];
            for p in candidates {
                if let Ok(data) = std::fs::read(p) {
                    fonts.font_data.insert(
                        "symbols".to_owned(),
                        egui::FontData::from_owned(data).into(),
                    );
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .push("symbols".to_owned());
                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("symbols".to_owned());
                    break;
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let candidates = [
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSansCondensed.ttf",
                "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
                "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
            ];
            for p in candidates {
                if let Ok(data) = std::fs::read(p) {
                    fonts.font_data.insert(
                        "symbols".to_owned(),
                        egui::FontData::from_owned(data).into(),
                    );
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .push("symbols".to_owned());
                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("symbols".to_owned());
                    break;
                }
            }
        }

        let font_paths = crate::ui::find_system_fonts();

        for path in font_paths {
            if let Ok(data) = std::fs::read(&path) {
                fonts.font_data.insert(
                    "system_cjk".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "system_cjk".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("system_cjk".to_owned());
                break;
            }
        }
        cc.egui_ctx.set_fonts(fonts);

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
            sort_column: None,
            sort_order: SortOrder::None,
            local_save_paths: Vec::new(),
            search_query: String::new(),
            show_only_local: false,
            show_only_cloud: false,
            multi_select_mode: false,
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

    fn sort_files(&mut self, column: SortColumn) {
        if self.sort_column == Some(column) {
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::None,
                SortOrder::None => SortOrder::Ascending,
            };
        } else {
            self.sort_column = Some(column);
            self.sort_order = SortOrder::Ascending;
        }

        if self.sort_order == SortOrder::None {
            self.sort_column = None;
            self.refresh_files();
        } else {
            let order = self.sort_order;
            self.files.sort_by(|a, b| {
                let result = match column {
                    SortColumn::Name => a.name.cmp(&b.name),
                    SortColumn::Size => a.size.cmp(&b.size),
                    SortColumn::Time => a.timestamp.cmp(&b.timestamp),
                };
                match order {
                    SortOrder::Ascending => result,
                    SortOrder::Descending => result.reverse(),
                    SortOrder::None => std::cmp::Ordering::Equal,
                }
            });
        }
    }

    fn open_local_save_folder(&self, path: &PathBuf) {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer").arg(path).spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(path).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        }
    }

    fn update_local_save_paths(&mut self) {
        use std::collections::HashMap;

        let mut path_map: HashMap<String, PathBuf> = HashMap::new();
        let app_id = match self.app_id_input.parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                self.local_save_paths.clear();
                return;
            }
        };

        let parser_data = self
            .ensure_vdf_parser()
            .map(|p| (p.get_steam_path().clone(), p.get_user_id().to_string()));

        for file in &self.files {
            if file.exists {
                if let Some((ref steam_path, ref user_id)) = parser_data {
                    // 使用 path_resolver 模块解析路径
                    if let Ok(path) = crate::path_resolver::resolve_cloud_file_path(
                        file.root, &file.name, steam_path, user_id, app_id,
                    ) {
                        if let Some(parent) = path.parent() {
                            let parent_path = parent.to_path_buf();
                            if parent_path.exists() {
                                let key = format!("{} ({})", file.root_description, file.root);
                                path_map.entry(key).or_insert(parent_path);
                            }
                        }
                    }
                }
            }
        }

        let mut paths: Vec<(String, PathBuf)> = path_map.into_iter().collect();
        paths.sort_by(|a, b| a.0.cmp(&b.0));

        self.local_save_paths = paths;

        if !self.local_save_paths.is_empty() {
            tracing::info!("检测到 {} 个本地存档路径", self.local_save_paths.len());
            for (desc, path) in &self.local_save_paths {
                tracing::debug!("  - {}: {}", desc, path.display());
            }
        } else {
            tracing::debug!("未找到本地存档路径");
        }
    }

    fn download_selected_file(&mut self) {
        let file_index = match self.get_selected_file_index() {
            Ok(idx) => idx,
            Err(e) => {
                self.handle_error(e);
                return;
            }
        };

        if self.selected_files.len() != 1 {
            self.show_error("请只选择一个文件");
            return;
        }
        // 只克隆必要的字段，而不是整个对象
        let filename = self.files[file_index].name.clone();
        let file_for_download = self.files[file_index].clone();

        if let Some(path) = FileDialog::new().set_file_name(&filename).save_file() {
            self.download_file_to_path(&file_for_download, &path);
        }
    }

    fn download_file_to_path(&mut self, file: &CloudFile, path: &PathBuf) {
        let url_prefix = "CDP:";
        if file.root_description.starts_with(url_prefix) {
            let content = &file.root_description[url_prefix.len()..];
            let url = content.split('|').next().unwrap_or("");

            tracing::info!("正在从 CDP 下载: {}", url);

            match ureq::get(url).call() {
                Ok(resp) => {
                    let mut reader = resp.into_reader();
                    let mut data = Vec::new();
                    if let Err(e) = std::io::Read::read_to_end(&mut reader, &mut data) {
                        self.show_error(&format!("读取响应流失败: {}", e));
                        return;
                    }

                    self.save_data_to_path(&data, path);
                }
                Err(e) => {
                    self.show_error(&format!("HTTP 下载失败: {}", e));
                }
            }
        } else {
            let result = match self.steam_manager.lock() {
                Ok(manager) => manager.read_file(&file.name),
                Err(e) => Err(anyhow::anyhow!("Steam 管理器锁错误: {}", e)),
            };

            match result {
                Ok(data) => {
                    self.save_data_to_path(&data, path);
                }
                Err(e) => {
                    self.show_error(&format!("API 下载失败 (可能文件未缓存): {}", e));
                }
            }
        }
    }

    fn save_data_to_path(&mut self, data: &[u8], path: &PathBuf) {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    self.show_error(&format!("创建目录失败: {}", e));
                    return;
                }
            }
        }

        match std::fs::write(path, data) {
            Ok(()) => {
                self.status_message = format!("文件已下载: {}", path.display());
            }
            Err(e) => {
                self.show_error(&format!("保存文件失败: {}", e));
            }
        }
    }

    fn upload_file(&mut self) {
        if !self.is_connected {
            self.show_error("未连接到Steam");
            return;
        }

        if let Some(path) = FileDialog::new().add_filter("所有文件", &["*"]).pick_file() {
            match std::fs::read(&path) {
                Ok(data) => {
                    let filename = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.replace('\\', "/"))
                        .unwrap_or("unknown_file".to_string());

                    let filename = filename.as_str();

                    let result = match self.steam_manager.lock() {
                        Ok(manager) => manager.write_file(filename, &data),
                        Err(e) => Err(anyhow::anyhow!("Steam 管理器锁错误: {}", e)),
                    };

                    match result {
                        Ok(true) => {
                            self.status_message = format!("文件已上传: {}", filename);
                            self.refresh_files();
                        }
                        Ok(false) => {
                            self.show_error("文件上传失败");
                        }
                        Err(e) => {
                            self.show_error(&format!("上传文件失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.show_error(&format!("读取文件失败: {}", e));
                }
            }
        }
    }

    fn forget_selected_files(&mut self) {
        if self.selected_files.is_empty() {
            self.show_error("请选择要取消云同步的文件");
            return;
        }

        let steam_manager = self.steam_manager.clone();
        let (forgotten_count, failed_files) = self.batch_file_operation(
            |filename| match steam_manager.lock() {
                Ok(manager) => manager.forget_file(filename),
                Err(e) => Err(anyhow::anyhow!("Steam 管理器锁错误: {}", e)),
            },
            "forget",
        );

        if !failed_files.is_empty() {
            self.show_error(&format!(
                "部分文件取消云同步失败: {}",
                failed_files.join(", ")
            ));
        }

        if forgotten_count > 0 {
            self.status_message = format!("已取消云同步 {} 个文件", forgotten_count);
            self.refresh_files();
        }
    }

    fn delete_selected_files(&mut self) {
        if self.selected_files.is_empty() {
            self.show_error("请选择要删除的文件");
            return;
        }

        let steam_manager = self.steam_manager.clone();
        let (deleted_count, failed_files) = self.batch_file_operation(
            |filename| match steam_manager.lock() {
                Ok(manager) => manager.delete_file(filename),
                Err(e) => Err(anyhow::anyhow!("Steam 管理器锁错误: {}", e)),
            },
            "delete",
        );

        if !failed_files.is_empty() {
            self.show_error(&format!("部分文件删除失败: {}", failed_files.join(", ")));
        }

        if deleted_count > 0 {
            self.status_message = format!("已删除 {} 个文件", deleted_count);
            self.refresh_files();
        }
    }

    fn show_error(&mut self, message: &str) {
        self.error_message = message.to_string();
        self.show_error = true;
    }

    fn fetch_and_merge_games(
        steam_path: PathBuf,
        user_id: String,
    ) -> Result<Vec<CloudGameInfo>, String> {
        let mut games = crate::game_scanner::scan_cloud_games(&steam_path, &user_id)
            .map_err(|e| e.to_string())?;

        let mut cdp_order = std::collections::HashMap::new();
        if crate::cdp_client::CdpClient::is_cdp_running() {
            if let Ok(mut client) = crate::cdp_client::CdpClient::connect() {
                if let Ok(cdp_games) = client.fetch_game_list() {
                    let map: std::collections::HashMap<u32, usize> = games
                        .iter()
                        .enumerate()
                        .map(|(i, g)| (g.app_id, i))
                        .collect();

                    let mut added = 0;
                    let mut updated = 0;

                    for (idx, cdp_game) in cdp_games.into_iter().enumerate() {
                        cdp_order.insert(cdp_game.app_id, idx);

                        if let Some(&i) = map.get(&cdp_game.app_id) {
                            let g = &mut games[i];
                            g.file_count = cdp_game.file_count;
                            g.total_size = cdp_game.total_size;
                            if let Some(name) = cdp_game.game_name {
                                if g.game_name.is_none() || g.game_name.as_deref() == Some("") {
                                    g.game_name = Some(name);
                                    updated += 1;
                                }
                            }
                        } else {
                            games.push(cdp_game);
                            added += 1;
                        }
                    }
                    tracing::info!(
                        "CDP 合并完成: 新增 {} 个游戏, 更新 {} 个信息",
                        added,
                        updated
                    );
                }
            }
        }

        // 排序：已安装 -> CDP顺序 -> 名称
        games.sort_by(|a, b| {
            if a.is_installed != b.is_installed {
                return b.is_installed.cmp(&a.is_installed);
            }

            match (cdp_order.get(&a.app_id), cdp_order.get(&b.app_id)) {
                (Some(ia), Some(ib)) => ia.cmp(ib),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => {
                    let name_a = a.game_name.as_deref().unwrap_or("");
                    let name_b = b.game_name.as_deref().unwrap_or("");
                    if name_a.is_empty() && name_b.is_empty() {
                        a.app_id.cmp(&b.app_id)
                    } else {
                        name_a.cmp(name_b)
                    }
                }
            }
        });

        Ok(games)
    }

    fn scan_cloud_games(&mut self) {
        // 先获取必要的数据，避免借用冲突
        let parser_data = self
            .ensure_vdf_parser()
            .map(|p| (p.get_steam_path().clone(), p.get_user_id().to_string()));

        if let Some((steam_path, user_id)) = parser_data {
            self.is_scanning_games = true;
            self.status_message = "正在扫描游戏库...".to_string();
            let (tx, rx) = std::sync::mpsc::channel();
            self.scan_games_rx = Some(rx);

            std::thread::spawn(move || {
                let result = Self::fetch_and_merge_games(steam_path, user_id);
                let _ = tx.send(result);
            });
        } else {
            self.show_error("VDF 解析器未初始化");
        }
    }

    fn draw_game_selector(&mut self, ctx: &egui::Context) {
        let selected_app_id = crate::ui::draw_game_selector_window(
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
    }

    fn handle_file_drop(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let painter = ui.painter();
            let rect = ui.available_rect_before_wrap();
            painter.rect_filled(
                rect,
                5.0,
                egui::Color32::from_rgba_premultiplied(0, 100, 200, 50),
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "",
                egui::FontId::proportional(20.0),
                egui::Color32::WHITE,
            );
        }

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

    fn upload_file_from_path(&mut self, path: &PathBuf) {
        if !path.is_file() {
            self.show_error("只能上传文件");
            return;
        }

        let filename = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                self.show_error("无法获取文件名");
                return;
            }
        };

        match std::fs::read(path) {
            Ok(data) => {
                let result = match self.steam_manager.lock() {
                    Ok(manager) => manager.write_file(&filename, &data),
                    Err(e) => Err(anyhow::anyhow!("Steam 管理器锁错误: {}", e)),
                };

                match result {
                    Ok(_) => {
                        self.status_message = format!("上传成功: {}", filename);
                        self.refresh_files();
                    }
                    Err(e) => {
                        self.show_error(&format!("上传失败: {}", e));
                    }
                }
            }
            Err(e) => {
                self.show_error(&format!("读取文件失败: {}", e));
            }
        }
    }

    fn load_all_users(&mut self) {
        if let Some(parser) = &self.vdf_parser {
            if let Ok(users) = parser.get_all_users_info() {
                self.all_users = users;
            }
        }
    }

    fn switch_user(&mut self, user_id: String) {
        if let Some(parser) = &self.vdf_parser {
            let steam_path = parser.get_steam_path().clone();
            self.vdf_parser = Some(VdfParser::with_user_id(steam_path, user_id));
            self.cloud_games.clear();
            self.status_message = "已切换用户".to_string();
            self.scan_cloud_games();
        }
    }

    fn draw_user_selector(&mut self, ctx: &egui::Context) {
        let selected_user_id = crate::ui::draw_user_selector_window(
            ctx,
            &mut self.show_user_selector,
            &self.all_users,
        );

        if let Some(user_id) = selected_user_id {
            self.switch_user(user_id);
            self.show_user_selector = false;
        }
    }
    fn draw_about_window(&mut self, ctx: &egui::Context) {
        let steam_blue = egui::Color32::from_rgb(102, 192, 244);
        let text_subtle = ctx.style().visuals.text_color().gamma_multiply(0.6);
        let text_normal = ctx.style().visuals.text_color();

        egui::Window::new("About")
            .open(&mut self.show_about)
            .resizable(false)
            .collapsible(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.add_space(16.0);

                ui.vertical_centered(|ui| {
                    // 加载应用图标
                    if self.about_icon_texture.is_none() {
                        let icon_bytes =
                            include_bytes!("../assets/steam_cloud-macOS-Default-1024x1024@1x.png");
                        if let Ok(img) = image::load_from_memory(icon_bytes) {
                            let img =
                                img.resize_exact(128, 128, image::imageops::FilterType::Lanczos3);
                            let rgba = img.to_rgba8();
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                [128, 128],
                                rgba.as_flat_samples().as_slice(),
                            );
                            self.about_icon_texture = Some(ui.ctx().load_texture(
                                "about_icon",
                                color_image,
                                Default::default(),
                            ));
                        }
                    }

                    if let Some(texture) = &self.about_icon_texture {
                        ui.image(texture);
                    }

                    ui.add_space(16.0);

                    ui.label(
                        egui::RichText::new("Steam Cloud File Manager")
                            .size(22.0)
                            .strong()
                            .color(text_normal),
                    );
                });

                ui.add_space(24.0);

                ui.horizontal(|ui| {
                    let width = ui.available_width();
                    let content_width = 280.0;
                    ui.add_space((width - content_width) / 2.0);

                    ui.vertical(|ui| {
                        ui.set_width(content_width);

                        egui::Grid::new("tech_grid")
                            .num_columns(2)
                            .spacing([24.0, 8.0])
                            .striped(false)
                            .show(ui, |ui| {
                                let mut row = |key: &str, val: String| {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(key)
                                                    .size(13.0)
                                                    .color(text_subtle),
                                            );
                                        },
                                    );
                                    ui.label(
                                        egui::RichText::new(val)
                                            .size(13.0)
                                            .color(text_normal)
                                            .monospace(),
                                    );
                                    ui.end_row();
                                };

                                row("Version", crate::version::full_version().to_string());
                            });
                    });
                });

                ui.add_space(24.0);

                ui.separator();
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    let width = ui.available_width();
                    let content_width = 380.0;
                    ui.add_space((width - content_width) / 2.0);

                    ui.vertical(|ui| {
                        ui.set_width(content_width);

                        egui::Grid::new("links_grid")
                            .num_columns(2)
                            .spacing([12.0, 8.0])
                            .show(ui, |ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new("Author:")
                                                .size(12.0)
                                                .color(text_subtle),
                                        );
                                    },
                                );
                                ui.hyperlink_to(
                                    egui::RichText::new("Flacier").size(12.0).color(steam_blue),
                                    "https://github.com/Fldicoahkiin",
                                );
                                ui.end_row();

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new("Repository:")
                                                .size(12.0)
                                                .color(text_subtle),
                                        );
                                    },
                                );
                                ui.hyperlink_to(
                                    egui::RichText::new(
                                        "https://github.com/Fldicoahkiin/SteamCloudFileManager",
                                    )
                                    .size(12.0)
                                    .color(steam_blue),
                                    "https://github.com/Fldicoahkiin/SteamCloudFileManager",
                                );
                                ui.end_row();
                            });
                    });
                });

                ui.add_space(16.0);

                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Copyright © 2025 Flacier")
                            .size(10.0)
                            .color(text_subtle),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("GPL-3.0 License")
                            .size(10.0)
                            .color(text_subtle),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("Powered by Rust & egui")
                            .size(10.0)
                            .color(text_subtle),
                    );
                });

                ui.add_space(10.0);
            });
    }
    fn draw_connection_panel(&mut self, ui: &mut egui::Ui) {
        if self.show_debug_warning {
            crate::ui::draw_debug_warning(ui, || {
                if let Err(e) = crate::steam_api::restart_steam_with_debugging() {
                    self.show_error(&format!("重启失败: {}", e));
                } else {
                    self.status_message = "正在重启 Steam...".to_string();
                }
            });
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
                self.load_all_users();
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
        if self.files.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("没有找到云文件");
            });
            return;
        }

        if !self.local_save_paths.is_empty() {
            ui.label("本地存档路径:");
            ui.horizontal_wrapped(|ui| {
                for (desc, path) in &self.local_save_paths {
                    let button_text = format!("📁 {}", desc);
                    if ui
                        .button(button_text)
                        .on_hover_text(path.display().to_string())
                        .clicked()
                    {
                        self.open_local_save_folder(path);
                    }
                }
            });
            ui.separator();
        } else if self.remote_ready {
            ui.horizontal(|ui| {
                ui.label("本地存档路径:");
                ui.label("未找到（可能所有文件都仅在云端）");
            });
            ui.separator();
        }

        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .desired_width(200.0)
                    .hint_text("搜索文件..."),
            );

            if ui.button("清除搜索").clicked() {
                self.search_query.clear();
            }

            ui.separator();

            if ui
                .selectable_label(self.show_only_local, "仅本地")
                .clicked()
            {
                self.show_only_local = !self.show_only_local;
                if self.show_only_local {
                    self.show_only_cloud = false;
                }
            }

            if ui
                .selectable_label(self.show_only_cloud, "仅云端")
                .clicked()
            {
                self.show_only_cloud = !self.show_only_cloud;
                if self.show_only_cloud {
                    self.show_only_local = false;
                }
            }

            if ui
                .selectable_label(self.multi_select_mode, "多选模式")
                .clicked()
            {
                self.multi_select_mode = !self.multi_select_mode;
            }
        });

        use egui_extras::{Column, TableBuilder};

        let available_height = ui.available_height();
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(150.0))
            .column(Column::remainder().at_least(150.0))
            .column(Column::exact(80.0))
            .column(Column::exact(160.0))
            .column(Column::exact(40.0))
            .column(Column::exact(40.0))
            .max_scroll_height(available_height)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("文件夹");
                });
                header.col(|ui| {
                    let name_btn = if self.sort_column == Some(SortColumn::Name) {
                        match self.sort_order {
                            SortOrder::Ascending => "文件名 ▲",
                            SortOrder::Descending => "文件名 ▼",
                            SortOrder::None => "文件名",
                        }
                    } else {
                        "文件名"
                    };
                    if ui.button(name_btn).clicked() {
                        self.sort_files(SortColumn::Name);
                    }
                });
                header.col(|ui| {
                    let size_btn = if self.sort_column == Some(SortColumn::Size) {
                        match self.sort_order {
                            SortOrder::Ascending => "文件大小 ▲",
                            SortOrder::Descending => "文件大小 ▼",
                            SortOrder::None => "文件大小",
                        }
                    } else {
                        "文件大小"
                    };
                    if ui.button(size_btn).clicked() {
                        self.sort_files(SortColumn::Size);
                    }
                });
                header.col(|ui| {
                    let time_btn = if self.sort_column == Some(SortColumn::Time) {
                        match self.sort_order {
                            SortOrder::Ascending => "写入日期 ▲",
                            SortOrder::Descending => "写入日期 ▼",
                            SortOrder::None => "写入日期",
                        }
                    } else {
                        "写入日期"
                    };
                    if ui.button(time_btn).clicked() {
                        self.sort_files(SortColumn::Time);
                    }
                });
                header.col(|ui| {
                    ui.label("本地");
                });
                header.col(|ui| {
                    ui.label("云端");
                });
            })
            .body(|body| {
                self.draw_file_items_table(body);
            });
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
                self.selected_files = (0..self.files.len()).collect();
            }
            crate::ui::FileAction::InvertSelection => {
                let current_selected: std::collections::HashSet<_> =
                    self.selected_files.iter().copied().collect();
                self.selected_files = (0..self.files.len())
                    .filter(|i| !current_selected.contains(i))
                    .collect();
            }
            crate::ui::FileAction::ClearSelection => {
                self.selected_files.clear();
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
        ui.separator();

        // 状态消息栏
        let cloud_enabled = if self.is_connected {
            self.steam_manager
                .lock()
                .ok()
                .and_then(|m| m.is_cloud_enabled_for_app().ok())
        } else {
            None
        };

        let toggled = crate::ui::draw_status_message(ui, &self.status_message, cloud_enabled);

        if toggled {
            if let Ok(manager) = self.steam_manager.lock() {
                if let Ok(enabled) = manager.is_cloud_enabled_for_app() {
                    let _ = manager.set_cloud_enabled_for_app(!enabled);
                }
            }
        }

        // 云存储状态
        if self.is_connected {
            if self.remote_ready {
                if let Ok(manager) = self.steam_manager.lock() {
                    let account_enabled = manager.is_cloud_enabled_for_account().ok();
                    let app_enabled = manager.is_cloud_enabled_for_app().ok();
                    crate::ui::draw_cloud_status(ui, account_enabled, app_enabled);
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label("云存储状态:");
                    ui.label("未就绪");
                });
            }
        }

        // 配额信息
        if let Some((total, available)) = self.quota_info {
            crate::ui::draw_quota_info(ui, total, available);
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
                    self.update_local_save_paths();
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
                Ok(Ok(games)) => {
                    self.cloud_games = games;
                    self.status_message =
                        format!("发现 {} 个有云存档的游戏", self.cloud_games.len());
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

        if self.show_error {
            egui::Window::new("错误")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&self.error_message);
                    if ui.button("确定").clicked() {
                        self.show_error = false;
                    }
                });
        }

        if self.show_game_selector {
            self.draw_game_selector(ctx);
        }

        if self.show_user_selector {
            self.draw_user_selector(ctx);
        }

        if self.show_about {
            self.draw_about_window(ctx);
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
