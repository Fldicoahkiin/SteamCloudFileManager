use crate::steam_api::{CloudFile, SteamCloudManager};
use eframe::egui;
use rfd::FileDialog;
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

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.columns(5, |columns| {
                columns[0].heading("文件名");
                columns[1].heading("大小");
                columns[2].heading("修改时间");
                columns[3].heading("已持久保存");
                columns[4].heading("云端存在");

                for (index, file) in self.files.iter().enumerate() {
                    let is_selected = self.selected_files.contains(&index);

                    if columns[0]
                        .selectable_label(is_selected, &file.name)
                        .clicked()
                    {
                        if is_selected {
                            self.selected_files.retain(|&x| x != index);
                        } else {
                            self.selected_files.push(index);
                        }
                    }

                    columns[1].label(Self::format_size(file.size));
                    columns[2].label(file.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
                    columns[3].label(if file.is_persisted { "是" } else { "否" });
                    columns[4].label(if file.exists { "是" } else { "否" });
                }
            });
        });
    }

    fn draw_action_buttons(&mut self, ui: &mut egui::Ui) {
        ui.separator();

        ui.horizontal(|ui| {
            let can_ops = self.is_connected
                && self.remote_ready
                && !self.is_refreshing
                && !self.is_connecting;
            if ui
                .add_enabled(can_ops, egui::Button::new("下载选中文件"))
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
                .add_enabled(can_ops, egui::Button::new("删除选中文件"))
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

            ui.label(format!("已选择 {} 个文件", self.selected_files.len()));
        });

        if self.is_connected {
            ui.horizontal(|ui| {
                if let Ok(manager) = self.steam_manager.lock() {
                    if let Ok(enabled) = manager.is_cloud_enabled_for_app() {
                        if ui
                            .button(if enabled {
                                "禁用应用云存储"
                            } else {
                                "启用应用云存储"
                            })
                            .clicked()
                        {
                            let _ = manager.set_cloud_enabled_for_app(!enabled);
                        }
                    }
                }
            });
        }
    }

    fn draw_status_panel(&mut self, ui: &mut egui::Ui) {
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("状态:");
            ui.label(&self.status_message);
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
