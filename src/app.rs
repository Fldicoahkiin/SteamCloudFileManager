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
    vdf_parser: Option<VdfParser>,
    all_users: Vec<UserInfo>,
    show_user_selector: bool,
    show_about: bool,
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
                    ui.label(&file.root_description);
                });

                row.col(|ui| {
                    if ui.selectable_label(is_selected, &file.name).clicked() {
                        if self.multi_select_mode {
                            if is_selected {
                                self.selected_files.retain(|&x| x != index);
                            } else {
                                self.selected_files.push(index);
                            }
                        } else {
                            self.selected_files.clear();
                            if !is_selected {
                                self.selected_files.push(index);
                            }
                        }
                    }
                });

                row.col(|ui| {
                    ui.label(Self::format_size(file.size));
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

    fn format_size(size: i32) -> String {
        let bytes = if size < 0 { 0.0 } else { size as f64 };
        if bytes < 1024.0 {
            format!("{} B", size.max(0))
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
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
                let symbols_path = std::path::PathBuf::from(&windir).join("Fonts").join("seguisym.ttf");
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

        Self {
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
            vdf_parser: VdfParser::new().ok(),
            all_users: Vec::new(),
            show_user_selector: false,
            show_about: false,
        }
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

        log::info!("开始刷新云文件列表...");
        self.is_refreshing = true;

        let result = {
            let mgr = self.steam_manager.lock().unwrap();
            mgr.get_files()
        };

        match result {
            Ok(files) => {
                let count = files.len();
                log::info!("成功获取 {} 个云文件", count);

                if count == 0 {
                    log::warn!("云文件列表为空，可能原因：");
                    log::warn!("1. 游戏确实没有云存档");
                    log::warn!("2. Steam API 还在初始化中，请等待几秒后重试");
                    log::warn!("3. 游戏的云同步功能未启用");
                }

                self.files = files;
                self.selected_files.clear();
                self.update_quota();
                self.update_local_save_paths();

                self.status_message = format!("已加载 {} 个文件", count);
                self.remote_ready = true;
            }
            Err(err) => {
                log::error!("刷新文件列表失败: {}", err);
                self.show_error(&format!("刷新文件列表失败: {}", err));
            }
        }

        self.is_refreshing = false;
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

        // 从已加载的文件中提取所有唯一的父目录路径
        let mut path_map: HashMap<String, PathBuf> = HashMap::new();

        for file in &self.files {
            // 从文件的root_description和实际存在性推断路径
            if file.exists {
                // 尝试通过VDF解析器获取实际路径
                if let Ok(app_id) = self.app_id_input.parse::<u32>() {
                    if let Ok(parser) = crate::vdf_parser::VdfParser::new() {
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
        }

        // 转换为Vec并排序
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
        let filename = self.files[file_index].name.clone();

        if let Some(path) = FileDialog::new().set_file_name(&filename).save_file() {
            self.download_file_to_path(&filename, &path);
        }
    }

    fn download_file_to_path(&mut self, filename: &str, path: &PathBuf) {
        let result = {
            let manager = self.steam_manager.lock().unwrap();
            manager.read_file(filename)
        };

        match result {
            Ok(data) => {
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
            Err(e) => {
                self.show_error(&format!("下载文件失败: {}", e));
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

    fn scan_cloud_games(&mut self) {
        if self.vdf_parser.is_none() {
            self.vdf_parser = VdfParser::new().ok();
        }
        if let Some(parser) = &self.vdf_parser {
            self.is_scanning_games = true;
            match parser.scan_all_cloud_games() {
                Ok(games) => {
                    self.cloud_games = games;
                    self.show_game_selector = true;
                    self.status_message =
                        format!("发现 {} 个有云存档的游戏", self.cloud_games.len());
                }
                Err(e) => {
                    self.show_error(&format!("扫描游戏失败: {}", e));
                }
            }
            self.is_scanning_games = false;
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
                                        Self::format_size_i64(game.total_size)
                                    ));

                                    if let Some(dir) = &game.install_dir {
                                        ui.label(format!("安装目录: {}", dir));
                                    }

                                    if !game.categories.is_empty() {
                                        ui.label(format!("标签: {}", game.categories.join(", ")));
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
            });

        if let Some(app_id) = selected_app_id {
            self.app_id_input = app_id.to_string();
            self.show_game_selector = false;
            self.connect_to_steam();
        }
    }

    fn format_size_i64(size: i64) -> String {
        let bytes = if size < 0 { 0.0 } else { size as f64 };
        if bytes < 1024.0 {
            format!("{} B", size.max(0))
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
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
            if let Ok(user_ids) = parser.get_all_user_ids() {
                let current_user = parser.get_user_id();
                self.all_users = user_ids
                    .into_iter()
                    .map(|id| UserInfo {
                        user_id: id.clone(),
                        persona_name: None,
                        is_current: id == current_user,
                    })
                    .collect();
            }
        }
    }

    fn switch_user(&mut self, user_id: String) {
        if let Some(parser) = &self.vdf_parser {
            let steam_path = parser.get_steam_path().clone();
            self.vdf_parser = Some(VdfParser::with_user_id(steam_path, user_id));
            self.cloud_games.clear();
            self.status_message = "已切换用户".to_string();
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
                                    ui.strong(format!("用户 ID: {}", user.user_id));
                                    if user.is_current {
                                        ui.label("✅ 当前用户");
                                    }
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if !user.is_current && ui.button("切换").clicked() {
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

                ui.label("License: MIT License");
                ui.add_space(5.0);
                ui.label("Copyright (c) 2025 Flacier");

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label("Platform Support:");
                ui.label("  Windows | macOS | Linux");

                ui.add_space(10.0);
                ui.label("Built with Rust and egui");
            });
    }

    fn draw_connection_panel(&mut self, ui: &mut egui::Ui) {
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
            }

            ui.separator();

            if let Some(parser) = &self.vdf_parser {
                ui.label(format!("用户: {}", parser.get_user_id()));
                ui.separator();
            }

            ui.label("App ID:");
            ui.add(egui::TextEdit::singleline(&mut self.app_id_input).desired_width(150.0));

            let connect_btn = ui.add_enabled(
                !self.is_connecting,
                egui::Button::new(if self.is_connecting {
                    "正在连接..."
                } else {
                    "连接"
                }),
            );
            if connect_btn.clicked() {
                self.connect_to_steam();
            }

            if self.is_connected {
                if ui.button("断开连接").clicked() {
                    self.disconnect_from_steam();
                }

                let ready = self
                    .since_connected
                    .map(|t| t.elapsed() >= Duration::from_millis(800))
                    .unwrap_or(false);
                let refresh_btn = ui.add_enabled(
                    !self.is_refreshing && ready,
                    egui::Button::new(if self.is_refreshing {
                        "刷新中..."
                    } else {
                        "刷新"
                    }),
                );
                if refresh_btn.clicked() {
                    self.refresh_files();
                }
                if !ready {
                    ui.label("准备云存储接口...");
                }
            }
        });
    }

    fn draw_file_list(&mut self, ui: &mut egui::Ui) {
        if self.is_refreshing {
            ui.centered_and_justified(|ui| {
                ui.label("正在刷新文件列表...");
            });
            return;
        }

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
            .column(Column::exact(150.0)) // 文件夹 - 固定宽度
            .column(Column::remainder().at_least(150.0)) // 文件名 - 可拉伸
            .column(Column::exact(80.0)) // 文件大小 - 固定宽度
            .column(Column::exact(160.0)) // 写入日期 - 固定宽度
            .column(Column::exact(40.0)) // 本地 - 固定宽度
            .column(Column::exact(40.0)) // 云端 - 固定宽度
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
                    let mut total_size = 0i32;
                    for &idx in &self.selected_files {
                        if let Some(file) = self.files.get(idx) {
                            total_size += file.size;
                        }
                    }
                    ui.label(format!("总大小: {}", Self::format_size(total_size)));
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
            // 仅在 RemoteStorage 就绪（成功刷新过一次）后才查询云存储状态，避免接口未就绪导致崩溃
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
                    ui.label("未就绪（请先点击刷新）");
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
                        log::info!("Steam API已准备就绪，自动刷新云文件列表");
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
                    self.status_message = format!(
                        "已连接到Steam (App ID: {})，请点击【刷新】加载云文件",
                        app_id
                    );
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
