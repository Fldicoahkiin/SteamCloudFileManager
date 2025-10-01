use crate::steam_api::{CloudFile, SteamCloudManager};
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

#[derive(PartialEq, Clone, Copy)]
enum SortOrder {
    Ascending,
    Descending,
    None,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::None
    }
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
    local_save_path: Option<PathBuf>,
    expanded_folders: std::collections::HashSet<String>,
    search_query: String,
    show_only_local: bool,
    show_only_cloud: bool,
    multi_select_mode: bool,
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
    fn draw_tree_items(&mut self, ui: &mut egui::Ui, parent_path: &str, indent: usize) {
        let mut folders = std::collections::HashMap::<String, Vec<usize>>::new();
        let mut direct_files = Vec::new();
        
        for (idx, file) in self.files.iter().enumerate() {
            if self.show_only_local && file.exists {
                continue;
            }
            if self.show_only_cloud && !file.exists {
                continue;
            }
            
            if !self.search_query.is_empty() {
                let query = self.search_query.to_lowercase();
                if !file.name.to_lowercase().contains(&query) {
                    continue;
                }
            }
            
            let relative_path = if parent_path.is_empty() {
                file.name.as_str()
            } else if file.name.starts_with(&format!("{}/", parent_path)) {
                &file.name[parent_path.len() + 1..]
            } else {
                continue;
            };
            
            if let Some(slash_pos) = relative_path.find('/') {
                let folder_name = relative_path[..slash_pos].to_string();
                let folder_path = if parent_path.is_empty() {
                    folder_name.clone()
                } else {
                    format!("{}/{}", parent_path, folder_name)
                };
                folders.entry(folder_path).or_insert_with(Vec::new).push(idx);
            } else {
                direct_files.push(idx);
            }
        }
        
        let mut sorted_folders: Vec<_> = folders.keys().cloned().collect();
        sorted_folders.sort();
        
        let dragging = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        
        for folder_path in sorted_folders {
            let folder_name = folder_path.split('/').last().unwrap_or(&folder_path);
            let indent_str = "  ".repeat(indent);
            
            let is_expanded = self.expanded_folders.contains(&folder_path);
            let arrow = if is_expanded { "▼" } else { "▶" };
            
            let label = format!("{}{} [文件夹] {}", indent_str, arrow, folder_name);
            let resp = ui.selectable_label(false, &label);
            
            if dragging && ui.rect_contains_pointer(resp.rect) {
                ui.painter().rect(
                    resp.rect,
                    3.0,
                    egui::Color32::from_rgba_unmultiplied(100, 149, 237, 30),
                    egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(100, 149, 237, 160)),
                );
            }
            
            if resp.clicked() {
                if is_expanded {
                    self.expanded_folders.remove(&folder_path);
                } else {
                    self.expanded_folders.insert(folder_path.clone());
                }
            }
            
            ui.label("-");
            ui.label("-");
            ui.label("-");
            ui.label("-");
            ui.end_row();
            
            if is_expanded {
                self.draw_tree_items(ui, &folder_path, indent + 1);
            }
        }
        
        for &index in &direct_files {
            let file = &self.files[index];
            let file_name = file.name.split('/').last().unwrap_or(&file.name);
            let indent_str = "  ".repeat(indent);
            
            let is_selected = self.selected_files.contains(&index);
            let label = format!("{}{}", indent_str, file_name);
            
            if ui.selectable_label(is_selected, &label).clicked() {
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
            
            ui.label(Self::format_size(file.size));
            ui.label(file.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
            ui.label(if file.is_persisted { "✓" } else { "✕" });
            ui.label(if file.exists { "✓" } else { "✕" });
            ui.end_row();
        }
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

        let font_paths = Self::find_system_fonts();

        for path in font_paths {
            if let Ok(data) = std::fs::read(&path) {
                fonts
                    .font_data
                    .insert("system_cjk".to_owned(), egui::FontData::from_owned(data));
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
            local_save_path: None,
            expanded_folders: std::collections::HashSet::new(),
            search_query: String::new(),
            show_only_local: false,
            show_only_cloud: false,
            multi_select_mode: false,
        }
    }

    fn connect_to_steam(&mut self) {
        if self.app_id_input.trim().is_empty() {
            self.show_error("请输入App ID");
            return;
        }

        if self.is_connecting {
            return;
        }

        match self.app_id_input.trim().parse::<u32>() {
            Ok(app_id) => {
                self.is_connecting = true;
                self.is_connected = false;
                self.remote_ready = false;
                self.files.clear();
                self.selected_files.clear();
                self.quota_info = None;
                self.status_message = "正在连接到 Steam...".to_string();

                let result = {
                    let mut manager = self.steam_manager.lock().unwrap();
                    manager.connect(app_id)
                };
                match result {
                    Ok(()) => {
                        self.is_connecting = false;
                        self.is_connected = true;
                        self.detect_local_save_path(app_id);
                        self.status_message =
                            format!("已连接到Steam (App ID: {})，正在加载云文件...", app_id);
                        self.since_connected = Some(Instant::now());
                        // 连接成功后自动刷新一次
                        self.auto_refresh_after_connect();
                    }
                    Err(e) => {
                        self.is_connecting = false;
                        self.show_error(&format!("连接Steam失败: {}", e));
                    }
                }
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

    fn auto_refresh_after_connect(&mut self) {
        // 连接后延迟一小段时间再自动刷新，确保 Steam API 完全就绪
        std::thread::sleep(Duration::from_millis(500));
        self.refresh_files_internal();
    }

    fn refresh_files(&mut self) {
        if !self.is_connected {
            self.show_error("未连接到Steam");
            return;
        }
        self.refresh_files_internal();
    }

    fn refresh_files_internal(&mut self) {
        self.is_refreshing = true;
        let result = {
            let mgr = self.steam_manager.lock().unwrap();
            mgr.get_files()
        };
        match result {
            Ok(files) => {
                let count = files.len();
                self.files = files;
                self.selected_files.clear();
                self.update_quota();
                self.status_message = format!("已加载 {} 个文件", count);
                self.remote_ready = true;
            }
            Err(err) => {
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

    fn open_local_save_folder(&self) {
        if let Some(path) = &self.local_save_path {
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
    }

    fn detect_local_save_path(&mut self, app_id: u32) {
        #[cfg(target_os = "windows")]
        {
            let possible_paths = vec![
                std::env::var("PROGRAMFILES(X86)")
                    .ok()
                    .map(|p| PathBuf::from(p).join("Steam")),
                std::env::var("PROGRAMFILES")
                    .ok()
                    .map(|p| PathBuf::from(p).join("Steam")),
                Some(PathBuf::from("C:\\Program Files (x86)\\Steam")),
                Some(PathBuf::from("C:\\Program Files\\Steam")),
            ];

            for steam_path in possible_paths.into_iter().flatten() {
                if let Ok(entries) = std::fs::read_dir(steam_path.join("userdata")) {
                    for entry in entries.flatten() {
                        let user_path = entry.path().join(format!("{}", app_id)).join("remote");
                        if user_path.exists() {
                            self.local_save_path = Some(user_path);
                            return;
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let steam_userdata = PathBuf::from(home.clone())
                    .join("Library/Application Support/Steam/userdata");
                
                if let Ok(entries) = std::fs::read_dir(&steam_userdata) {
                    for entry in entries.flatten() {
                        if let Some(dir_name) = entry.path().file_name() {
                            if dir_name.to_string_lossy().chars().all(|c| c.is_ascii_digit()) {
                                let user_path = entry.path().join(format!("{}", app_id)).join("remote");
                                if user_path.exists() {
                                    self.local_save_path = Some(user_path);
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let steam_userdata = PathBuf::from(home)
                    .join(".steam/steam/userdata");
                    
                if let Ok(entries) = std::fs::read_dir(&steam_userdata) {
                    for entry in entries.flatten() {
                        if let Some(dir_name) = entry.path().file_name() {
                            if dir_name.to_string_lossy().chars().all(|c| c.is_ascii_digit()) {
                                let user_path = entry.path().join(format!("{}", app_id)).join("remote");
                                if user_path.exists() {
                                    self.local_save_path = Some(user_path);
                                    return;
                                }
                            }
                        }
                    }
                }
            }
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
                    // Windows下确保文件名的正确处理
                    let filename = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| {
                            // 移除Windows路径分隔符
                            name.replace('\\', "/")
                        })
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

    fn draw_connection_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("App ID:");
            ui.text_edit_singleline(&mut self.app_id_input);

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

                if self.local_save_path.is_some() && ui.button("打开本地存档目录").clicked()
                {
                    self.open_local_save_folder();
                }
            }
        });
    }

    fn draw_file_list(&mut self, ui: &mut egui::Ui) {
        ui.separator();

        if self.is_refreshing {
            ui.label("正在刷新文件列表...");
            return;
        }

        if self.files.is_empty() {
            ui.label("没有找到云文件");
            return;
        }

        ui.horizontal(|ui| {
            if let Some(ref local_path) = self.local_save_path {
                ui.label(format!("本地存档路径: {}", local_path.display()));
                if ui.button("打开文件夹").clicked() {
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open").arg(local_path).spawn();
                    }
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("explorer").arg(local_path).spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("xdg-open").arg(local_path).spawn();
                    }
                }
            } else {
                ui.label("本地存档路径: 未找到");
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("清除搜索").clicked() {
                    self.search_query.clear();
                }
                ui.add(egui::TextEdit::singleline(&mut self.search_query).desired_width(200.0).hint_text("搜索文件..."));
                
                if ui.selectable_label(self.show_only_local, "仅本地").clicked() {
                    self.show_only_local = !self.show_only_local;
                    if self.show_only_local {
                        self.show_only_cloud = false;
                    }
                }
                
                if ui.selectable_label(self.show_only_cloud, "仅云端").clicked() {
                    self.show_only_cloud = !self.show_only_cloud;
                    if self.show_only_cloud {
                        self.show_only_local = false;
                    }
                }
                
                if ui.selectable_label(self.multi_select_mode, "多选模式").clicked() {
                    self.multi_select_mode = !self.multi_select_mode;
                }
            });
        });

        let scroll_area = egui::ScrollArea::vertical();
        let scroll_output = scroll_area.show(ui, |ui| {
            egui::Grid::new("file_grid")
                .num_columns(5)
                .striped(true)
                .min_col_width(60.0)
                .show(ui, |ui| {
                    if ui.button(if self.sort_column == Some(SortColumn::Name) {
                        match self.sort_order {
                            SortOrder::Ascending => "文件名 ▲",
                            SortOrder::Descending => "文件名 ▼",
                            SortOrder::None => "文件名",
                        }
                    } else {
                        "文件名"
                    }).clicked() {
                        self.sort_files(SortColumn::Name);
                    }
                    
                    if ui.button(if self.sort_column == Some(SortColumn::Size) {
                        match self.sort_order {
                            SortOrder::Ascending => "大小 ▲",
                            SortOrder::Descending => "大小 ▼",
                            SortOrder::None => "大小",
                        }
                    } else {
                        "大小"
                    }).clicked() {
                        self.sort_files(SortColumn::Size);
                    }
                    
                    if ui.button(if self.sort_column == Some(SortColumn::Time) {
                        match self.sort_order {
                            SortOrder::Ascending => "修改时间 ▲",
                            SortOrder::Descending => "修改时间 ▼",
                            SortOrder::None => "修改时间",
                        }
                    } else {
                        "修改时间"
                    }).clicked() {
                        self.sort_files(SortColumn::Time);
                    }
                    
                    ui.label("保存");
                    ui.label("云端已保存");
                    ui.end_row();

                    self.draw_tree_items(ui, "", 0);
                });
        });
        
        if ui.ctx().input(|i| !i.raw.hovered_files.is_empty()) 
            && ui.rect_contains_pointer(scroll_output.inner_rect) {
            ui.painter().rect(
                scroll_output.inner_rect,
                5.0,
                egui::Color32::from_rgba_unmultiplied(100, 149, 237, 30),
                egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(100, 149, 237, 100)),
            );
        }
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
                .add_enabled(can_ops && !self.selected_files.is_empty(), egui::Button::new("下载选中"))
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
                .add_enabled(can_ops && !self.selected_files.is_empty(), egui::Button::new("删除选中"))
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
                            let cloud_status = if enabled { "云存储: 开启" } else { "云存储: 关闭" };
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
        }

        if let Some(rx) = &self.connect_rx {
            match rx.try_recv() {
                Ok(Ok(app_id)) => {
                    self.is_connecting = false;
                    self.is_connected = true;
                    self.detect_local_save_path(app_id);
                    self.status_message =
                        format!("已连接到Steam (App ID: {})，请点击“刷新”加载云文件", app_id);
                    self.connect_rx = None;
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Steam 云文件管理器");

            self.draw_connection_panel(ui);

            if self.is_connected && self.remote_ready {
                self.handle_file_drop(ctx, ui);
            }

            self.draw_file_list(ui);
            self.draw_action_buttons(ui);
            self.draw_status_panel(ui);
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

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
