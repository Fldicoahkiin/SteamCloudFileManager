use crate::steam_api::{CloudFile, SteamCloudManager};
use crate::vdf_parser::{CloudGameInfo, UserInfo, VdfParser};
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
}

impl SteamCloudApp {
    fn find_system_fonts() -> Vec<std::path::PathBuf> {
        let mut font_paths = Vec::new();

        #[cfg(target_os = "macos")]
        {
            let home_font = format!(
                "{}/Library/Fonts",
                std::env::var("HOME").unwrap_or_default()
            );
            let dirs = vec![
                "/System/Library/Fonts",
                "/Library/Fonts",
                home_font.as_str(),
            ];

            for dir in dirs {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("ttf") {
                            font_paths.push(path);
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let mut font_dirs = Vec::new();

            if let Ok(windir) = std::env::var("WINDIR") {
                font_dirs.push(PathBuf::from(format!("{}/Fonts", windir)));
            }

            if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
                font_dirs.push(PathBuf::from(format!(
                    "{}\\Microsoft\\Windows\\Fonts",
                    localappdata
                )));
            }

            for font_dir in font_dirs {
                if let Ok(entries) = std::fs::read_dir(&font_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_str().unwrap_or("").to_lowercase();
                            if ext_str == "ttf" || ext_str == "ttc" || ext_str == "otf" {
                                font_paths.push(path);
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let mut font_dirs = Vec::new();
            font_dirs.push("/usr/share/fonts".to_string());
            font_dirs.push("/usr/local/share/fonts".to_string());
            font_dirs.push("/usr/share/fonts/truetype".to_string());

            if let Ok(home) = std::env::var("HOME") {
                font_dirs.push(format!("{}/.fonts", home));
                font_dirs.push(format!("{}/.local/share/fonts", home));
            }

            for dir in font_dirs {
                if let Ok(walker) = std::fs::read_dir(&dir) {
                    for entry in walker.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            if let Ok(sub_entries) = std::fs::read_dir(&path) {
                                for sub_entry in sub_entries.flatten() {
                                    let sub_path = sub_entry.path();
                                    if sub_path.extension().and_then(|s| s.to_str()) == Some("ttf")
                                    {
                                        font_paths.push(sub_path);
                                    }
                                }
                            }
                        } else if path.extension().and_then(|s| s.to_str()) == Some("ttf") {
                            font_paths.push(path);
                        }
                    }
                }
            }
        }

        font_paths.sort_by_key(|p| {
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            #[cfg(target_os = "windows")]
            {
                if name.contains("msyh") || name.contains("microsoft yahei") {
                    0
                } else if name.contains("simsun") {
                    1
                } else if name.contains("simhei") {
                    2
                } else if name.contains("arial") {
                    3
                } else if name.contains("segoe") {
                    4
                } else if name.contains("noto") && name.contains("cjk") {
                    5
                } else {
                    100
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                if name.contains("msyh") || name.contains("microsoft yahei") {
                    0
                } else if name.contains("simhei") || name.contains("heiti") {
                    1
                } else if name.contains("arial") {
                    2
                } else if name.contains("noto") && name.contains("cjk") {
                    3
                } else if name.contains("sarasa") {
                    4
                } else if name.contains("source") && name.contains("han") {
                    5
                } else if name.contains("wenquanyi") {
                    10
                } else {
                    100
                }
            }
        });

        font_paths
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
                    ui.label(Self::format_size_u64(file.size));
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

    fn format_size_u64(size: u64) -> String {
        let bytes = size as f64;
        if bytes < 1024.0 {
            format!("{} B", size)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
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

        let font_paths = Self::find_system_fonts();

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
        };

        if let Some(parser) = &app.vdf_parser {
            app.is_scanning_games = true;
            let (tx, rx) = std::sync::mpsc::channel();
            app.scan_games_rx = Some(rx);
            let steam_path = parser.get_steam_path().clone();
            let user_id = parser.get_user_id().to_string();
            std::thread::spawn(move || {
                let result = Self::fetch_and_merge_games(steam_path, user_id);
                let _ = tx.send(result);
            });
        }

        app
    }

    fn connect_to_steam(&mut self) {
        if self.app_id_input.trim().is_empty() {
            self.show_error("请输入App ID");
            return;
        }

        if self.is_connecting || self.connect_rx.is_some() {
            log::warn!("正在连接中，请勿重复点击");
            return;
        }

        match self.app_id_input.trim().parse::<u32>() {
            Ok(app_id) => {
                log::info!("开始连接到 Steam，App ID: {}", app_id);
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
                        let mut manager = steam_manager.lock().unwrap();
                        manager.connect(app_id)
                    };
                    let _ = tx.send(result.map(|_| app_id).map_err(|e| e.to_string()));
                });
            }
            Err(_) => {
                self.show_error("请输入有效的App ID");
            }
        }
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
        if !self.is_connected {
            self.show_error("未连接到Steam");
            return;
        }

        if self.loader_rx.is_some() {
            log::warn!("正在刷新中...");
            return;
        }

        log::info!("开始刷新云文件列表...");
        self.is_refreshing = true;
        self.files.clear();

        let steam_manager = self.steam_manager.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.loader_rx = Some(rx);

        let app_id = self.app_id_input.trim().parse::<u32>().unwrap_or(0);

        std::thread::spawn(move || {
            let mut files = {
                let mgr = steam_manager.lock().unwrap();
                mgr.get_files().unwrap_or_default()
            };

            // CDP 数据合并
            if crate::cdp_client::CdpClient::is_cdp_running() && app_id > 0 {
                log::info!("尝试通过 CDP 获取文件列表...");
                if let Ok(mut client) = crate::cdp_client::CdpClient::connect() {
                    if let Ok(cdp_files) = client.fetch_game_files(app_id) {
                        log::info!("CDP 返回 {} 个文件", cdp_files.len());
                        let file_map: std::collections::HashMap<String, usize> = files
                            .iter()
                            .enumerate()
                            .map(|(i, f)| (f.name.clone(), i))
                            .collect();

                        for cdp_file in cdp_files {
                            if let Some(&idx) = file_map.get(&cdp_file.name) {
                                let f = &mut files[idx];
                                f.size = cdp_file.size;
                                f.timestamp = cdp_file.timestamp;
                                f.is_persisted = true;
                                if cdp_file.root_description.starts_with("CDP:") {
                                    f.root_description = cdp_file.root_description;
                                }
                            } else {
                                files.push(cdp_file);
                            }
                        }
                    }
                }
            }

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

        if self.vdf_parser.is_none() {
            self.vdf_parser = VdfParser::new().ok();
        }
        let parser_opt = self.vdf_parser.as_ref();

        for file in &self.files {
            if file.exists {
                if let Some(parser) = parser_opt {
                    if let Ok(path) = parser.resolve_path(file.root, &file.name, app_id) {
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
            log::info!("检测到 {} 个本地存档路径", self.local_save_paths.len());
            for (desc, path) in &self.local_save_paths {
                log::info!("  - {}: {}", desc, path.display());
            }
        } else {
            log::warn!("未找到本地存档路径");
        }
    }

    fn download_selected_file(&mut self) {
        if self.selected_files.len() != 1 {
            self.show_error("请选择一个文件进行下载");
            return;
        }

        let file_index = self.selected_files[0];
        let file = self.files[file_index].clone();
        let filename = file.name.clone();

        if let Some(path) = FileDialog::new().set_file_name(&filename).save_file() {
            self.download_file_to_path(&file, &path);
        }
    }

    fn download_file_to_path(&mut self, file: &CloudFile, path: &PathBuf) {
        let url_prefix = "CDP:";
        if file.root_description.starts_with(url_prefix) {
            let content = &file.root_description[url_prefix.len()..];
            let url = content.split('|').next().unwrap_or("");

            log::info!("正在从 CDP 下载: {}", url);

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
            let result = {
                let manager = self.steam_manager.lock().unwrap();
                manager.read_file(&file.name)
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

                    let result = {
                        let manager = self.steam_manager.lock().unwrap();
                        manager.write_file(filename, &data)
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

        let filenames: Vec<String> = self
            .selected_files
            .iter()
            .map(|&index| self.files[index].name.clone())
            .collect();

        let mut forgotten_count = 0;
        let mut failed_files = Vec::new();

        for filename in &filenames {
            let result = {
                let manager = self.steam_manager.lock().unwrap();
                manager.forget_file(filename)
            };

            match result {
                Ok(true) => {
                    forgotten_count += 1;
                }
                Ok(false) => {
                    failed_files.push(filename.clone());
                }
                Err(e) => {
                    failed_files.push(format!("{} (错误: {})", filename, e));
                }
            }
        }

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

        let filenames: Vec<String> = self
            .selected_files
            .iter()
            .map(|&index| self.files[index].name.clone())
            .collect();

        let mut deleted_count = 0;
        let mut failed_files = Vec::new();

        for filename in &filenames {
            let result = {
                let manager = self.steam_manager.lock().unwrap();
                manager.delete_file(filename)
            };

            match result {
                Ok(true) => {
                    deleted_count += 1;
                }
                Ok(false) => {
                    failed_files.push(filename.clone());
                }
                Err(e) => {
                    failed_files.push(format!("{} (错误: {})", filename, e));
                }
            }
        }

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

    // 统一的游戏扫描逻辑：VDF扫描 + CDP数据获取与合并 + 排序
    fn fetch_and_merge_games(
        steam_path: PathBuf,
        user_id: String,
    ) -> Result<Vec<CloudGameInfo>, String> {
        let parser = VdfParser::with_user_id(steam_path, user_id);
        let mut games = parser.scan_all_cloud_games().map_err(|e| e.to_string())?;

        let mut cdp_order = std::collections::HashMap::new();
        if crate::cdp_client::CdpClient::is_cdp_running() {
            log::info!("检测到 CDP，尝试获取游戏列表...");
            if let Ok(mut client) = crate::cdp_client::CdpClient::connect() {
                if let Ok(cdp_games) = client.fetch_game_list() {
                    log::info!("CDP 返回 {} 个游戏", cdp_games.len());
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
                    log::info!(
                        "CDP 合并: 新增 {}, 更新 {}, 总计 {}",
                        added,
                        updated,
                        games.len()
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

        log::info!("游戏扫描完成，共 {} 个", games.len());
        Ok(games)
    }

    fn scan_cloud_games(&mut self) {
        if self.vdf_parser.is_none() {
            self.vdf_parser = VdfParser::new().ok();
        }
        if let Some(parser) = &self.vdf_parser {
            self.is_scanning_games = true;
            self.status_message = "正在扫描游戏库...".to_string();
            let (tx, rx) = std::sync::mpsc::channel();
            self.scan_games_rx = Some(rx);
            let steam_path = parser.get_steam_path().clone();
            let user_id = parser.get_user_id().to_string();

            std::thread::spawn(move || {
                let result = Self::fetch_and_merge_games(steam_path, user_id);
                let _ = tx.send(result);
            });
        } else {
            self.show_error("VDF 解析器未初始化");
        }
    }

    fn draw_game_selector(&mut self, ctx: &egui::Context) {
        let games = self.cloud_games.clone();
        let mut selected_app_id = None;

        egui::Window::new("游戏库")
            .open(&mut self.show_game_selector)
            .resizable(true)
            .default_size([600.0, 500.0])
            .show(ctx, |ui| {
                if self.is_scanning_games && games.is_empty() {
                    ui.label("正在扫描游戏库...");
                } else if games.is_empty() {
                    ui.label("未发现云存档的游戏");
                } else {
                    ui.heading(format!("{} 个有云存档的游戏", games.len()));
                    ui.add_space(10.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for game in &games {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            if let Some(name) = &game.game_name {
                                                ui.strong(name);
                                                if game.is_installed {
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(0, 200, 0),
                                                        "已安装",
                                                    );
                                                } else {
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(150, 150, 150),
                                                        "未安装",
                                                    );
                                                }
                                            } else {
                                                ui.strong(format!("App ID: {}", game.app_id));
                                                if game.is_installed {
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(0, 200, 0),
                                                        "已安装",
                                                    );
                                                } else {
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(150, 150, 150),
                                                        "未安装",
                                                    );
                                                }
                                            }
                                        });

                                        if game.game_name.is_some() {
                                            ui.label(format!("App ID: {}", game.app_id));
                                        }

                                        ui.label(format!(
                                            "{} 个文件 | {}",
                                            game.file_count,
                                            Self::format_size_u64(game.total_size)
                                        ));

                                        if let Some(dir) = &game.install_dir {
                                            ui.label(format!("安装目录: {}", dir));
                                        }

                                        if !game.categories.is_empty() {
                                            ui.label(format!(
                                                "标签: {}",
                                                game.categories.join(", ")
                                            ));
                                        }

                                        if let Some(playtime) = game.playtime {
                                            let hours = playtime / 60;
                                            ui.label(format!("游戏时间: {:.2} 小时", hours as f64));
                                        }

                                        if let Some(last_played) = game.last_played {
                                            if last_played > 0 {
                                                use chrono::{DateTime, Local};
                                                use std::time::{Duration, UNIX_EPOCH};
                                                let dt = UNIX_EPOCH
                                                    + Duration::from_secs(last_played as u64);
                                                let local: DateTime<Local> = dt.into();
                                                ui.label(format!(
                                                    "最后运行: {}",
                                                    local.format("%Y-%m-%d %H:%M")
                                                ));
                                            }
                                        }
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("选择").clicked() {
                                                selected_app_id = Some(game.app_id);
                                            }
                                        },
                                    );
                                });
                            });

                            ui.add_space(5.0);
                        }
                    });
                }
            });

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
                let dropped_files = i.raw.dropped_files.clone();
                for file in dropped_files {
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
                let result = {
                    let manager = self.steam_manager.lock().unwrap();
                    manager.write_file(&filename, &data)
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
        let users = self.all_users.clone();
        let mut selected_user = None;

        egui::Window::new("选择用户")
            .open(&mut self.show_user_selector)
            .resizable(true)
            .default_size([400.0, 300.0])
            .show(ctx, |ui| {
                ui.heading(format!("{} 个Steam用户", users.len()));
                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for user in &users {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    if let Some(name) = &user.persona_name {
                                        ui.strong(name);
                                        ui.label(format!("ID: {}", user.user_id));
                                    } else {
                                        ui.strong(format!("用户 ID: {}", user.user_id));
                                    }
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if user.is_current {
                                            ui.label("✅ 当前用户");
                                        } else if ui.button("切换").clicked() {
                                            selected_user = Some(user.user_id.clone());
                                        }
                                    },
                                );
                            });
                        });
                        ui.add_space(5.0);
                    }
                });
            });

        if let Some(user_id) = selected_user {
            self.switch_user(user_id);
            self.show_user_selector = false;
        }
    }

    fn draw_about_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .open(&mut self.show_about)
            .resizable(false)
            .collapsible(false)
            .default_width(450.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Steam Cloud File Manager");
                    ui.add_space(10.0);
                    ui.label("Version 1.0.0");
                    ui.add_space(15.0);
                });

                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Author:");
                    ui.hyperlink_to("Flacier", "https://github.com/Fldicoahkiin");
                });

                ui.horizontal(|ui| {
                    ui.label("Repository:");
                    ui.hyperlink_to(
                        "GitHub",
                        "https://github.com/Fldicoahkiin/SteamCloudFileManager",
                    );
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label("License: GPL-3.0 License");
                ui.add_space(5.0);
                ui.label("Copyright (c) 2025 Flacier");

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.add_space(10.0);
                ui.label("Built with Rust and egui");
            });
    }

    fn draw_connection_panel(&mut self, ui: &mut egui::Ui) {
        if self.show_debug_warning {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⚠ 注意：未检测到 Steam 调试模式")
                        .color(egui::Color32::YELLOW),
                );
                if ui.button("重启 Steam (开启调试)").clicked() {
                    if let Err(e) = crate::steam_api::restart_steam_with_debugging() {
                        self.show_error(&format!("重启失败: {}", e));
                    } else {
                        self.status_message = "正在重启 Steam...".to_string();
                    }
                }
            });
        }

        ui.horizontal(|ui| {
            if ui.button("关于").clicked() {
                self.show_about = true;
            }

            ui.separator();

            if ui.button("用户").clicked() {
                self.load_all_users();
                self.show_user_selector = true;
            }

            if ui.button("游戏库").clicked() {
                self.scan_cloud_games();
                self.show_game_selector = true;
            }

            ui.separator();

            if let Some(parser) = &self.vdf_parser {
                ui.label(format!("用户: {}", parser.get_user_id()));
                ui.separator();
            }

            ui.label("App ID:");
            let response =
                ui.add(egui::TextEdit::singleline(&mut self.app_id_input).desired_width(150.0));
            if response.changed() {
                self.is_connected = false;
                self.remote_ready = false;
                self.disconnect_from_steam();
            }

            if self.is_connected {
                if ui.button("断开").clicked() {
                    self.disconnect_from_steam();
                }
            } else if ui.button("连接").clicked() {
                self.connect_to_steam();
            }

            if self.is_connecting {
                ui.spinner();
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

        ui.horizontal(|ui| {
            let can_ops = self.is_connected
                && self.remote_ready
                && !self.is_refreshing
                && !self.is_connecting;

            if ui.button("全选").clicked() {
                self.selected_files.clear();
                for i in 0..self.files.len() {
                    self.selected_files.push(i);
                }
            }

            if ui.button("反选").clicked() {
                let current_selected = self.selected_files.clone();
                self.selected_files.clear();
                for i in 0..self.files.len() {
                    if !current_selected.contains(&i) {
                        self.selected_files.push(i);
                    }
                }
            }

            if ui.button("清除选择").clicked() {
                self.selected_files.clear();
            }

            ui.separator();

            if ui
                .add_enabled(
                    can_ops && !self.selected_files.is_empty(),
                    egui::Button::new("下载选中"),
                )
                .clicked()
            {
                self.download_selected_file();
            }

            if ui
                .add_enabled(can_ops, egui::Button::new("上传文件"))
                .clicked()
            {
                self.upload_file();
            }

            if ui
                .add_enabled(
                    can_ops && !self.selected_files.is_empty(),
                    egui::Button::new("删除选中"),
                )
                .clicked()
            {
                self.delete_selected_files();
            }

            if ui
                .add_enabled(can_ops, egui::Button::new("取消云同步"))
                .clicked()
            {
                self.forget_selected_files();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let selected_count = self.selected_files.len();
                let total_count = self.files.len();
                ui.label(format!("已选: {}/{}", selected_count, total_count));

                if selected_count > 0 {
                    let mut total_size: u64 = 0;
                    for &idx in &self.selected_files {
                        if let Some(file) = self.files.get(idx) {
                            total_size = total_size.saturating_add(file.size);
                        }
                    }
                    ui.label(format!("总大小: {}", Self::format_size_u64(total_size)));
                }
            });
        });
    }

    fn draw_status_panel(&mut self, ui: &mut egui::Ui) {
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("状态:");
            ui.label(&self.status_message);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.is_connected {
                    if let Ok(manager) = self.steam_manager.lock() {
                        if let Ok(enabled) = manager.is_cloud_enabled_for_app() {
                            let cloud_status = if enabled {
                                "云存储: 开启"
                            } else {
                                "云存储: 关闭"
                            };
                            if ui.selectable_label(false, cloud_status).clicked() {
                                let _ = manager.set_cloud_enabled_for_app(!enabled);
                            }
                        }
                    }
                }
            });
        });

        if self.is_connected {
            if self.remote_ready {
                if let Ok(manager) = self.steam_manager.lock() {
                    ui.horizontal(|ui| {
                        ui.label("账户云存储:");
                        match manager.is_cloud_enabled_for_account() {
                            Ok(enabled) => ui.label(if enabled {
                                "✅ 已启用"
                            } else {
                                "❌ 已禁用"
                            }),
                            Err(_) => ui.label("❓ 未知"),
                        };
                    });

                    ui.horizontal(|ui| {
                        ui.label("应用云存储:");
                        match manager.is_cloud_enabled_for_app() {
                            Ok(enabled) => ui.label(if enabled {
                                "✅ 已启用"
                            } else {
                                "❌ 已禁用"
                            }),
                            Err(_) => ui.label("❓ 未知"),
                        };
                    });
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label("云存储状态:");
                    ui.label("未就绪");
                });
            }
        }

        if let Some((total, available)) = self.quota_info {
            ui.horizontal(|ui| {
                ui.label("配额:");
                let used = total - available;
                let usage_percent = (used as f32 / total as f32 * 100.0).round();
                let used_str = Self::format_size_u64(used);
                let total_str = Self::format_size_u64(total);
                ui.label(format!(
                    "{:.1}% 已使用 ({}/{})",
                    usage_percent, used_str, total_str
                ));
            });
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
                        log::info!("Steam API已准备就绪");
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
                    log::info!("Steam连接成功");
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
