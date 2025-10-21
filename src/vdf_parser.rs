use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

pub struct VdfParser {
    steam_path: PathBuf,
    user_id: String,
}

#[derive(Debug, Clone)]
pub struct VdfFileEntry {
    pub filename: String,
    pub root: u32,
    pub size: u64,
    pub timestamp: i64,
    pub sha: String,
    pub sync_state: i32,
    pub actual_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudGameInfo {
    pub app_id: u32,
    pub file_count: usize,
    pub total_size: u64,
    pub last_played: Option<i64>,
    pub playtime: Option<u32>,
    pub game_name: Option<String>,
    pub is_installed: bool,
    pub install_dir: Option<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub app_id: u32,
    pub last_played: Option<i64>,
    pub playtime: Option<u32>,
    pub launch_options: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCategory {
    pub app_id: u32,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    pub app_id: u32,
    pub name: String,
    pub install_dir: String,
    pub size_on_disk: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub app_id: u32,
    pub name: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
    #[allow(dead_code)]
    pub persona_name: Option<String>,
    pub is_current: bool,
}

impl VdfParser {
    pub fn new() -> Result<Self> {
        let steam_path = Self::find_steam_path()?;
        let user_id = Self::find_user_id(&steam_path)?;
        Ok(Self {
            steam_path,
            user_id,
        })
    }

    fn find_steam_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let mut candidates: Vec<PathBuf> = Vec::new();
            if let Ok(p) = std::env::var("STEAM_PATH") {
                candidates.push(PathBuf::from(p));
            }
            if let Ok(p) = std::env::var("PROGRAMFILES(X86)") {
                candidates.push(PathBuf::from(p).join("Steam"));
            }
            if let Ok(p) = std::env::var("PROGRAMFILES") {
                candidates.push(PathBuf::from(p).join("Steam"));
            }
            if let Ok(p) = std::env::var("LOCALAPPDATA") {
                candidates.push(PathBuf::from(p).join("Steam"));
            }
            if let Ok(p) = std::env::var("APPDATA") {
                candidates.push(PathBuf::from(p).join("Steam"));
            }
            for c in candidates {
                if c.join("userdata").exists() || c.join("steam.exe").exists() {
                    return Ok(c);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")?;
            let path = PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("Steam");

            if path.exists() {
                return Ok(path);
            }
        }

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME")?;
            let paths = vec![
                PathBuf::from(&home).join(".steam").join("steam"),
                PathBuf::from(&home)
                    .join(".local")
                    .join("share")
                    .join("Steam"),
            ];

            for path in paths {
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        Err(anyhow!("未找到Steam安装目录"))
    }

    fn find_user_id(steam_path: &Path) -> Result<String> {
        if let Some(uid) = Self::find_user_id_from_loginusers(steam_path) {
            return Ok(uid);
        }
        let userdata_path = steam_path.join("userdata");
        if let Ok(entries) = fs::read_dir(&userdata_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.chars().all(|c| c.is_ascii_digit()) {
                    return Ok(name);
                }
            }
        }
        Err(anyhow!("未找到用户ID"))
    }

    fn find_user_id_from_loginusers(steam_path: &Path) -> Option<String> {
        let p = steam_path.join("config").join("loginusers.vdf");
        let s = fs::read_to_string(&p).ok()?;
        let mut current_id64: Option<u64> = None;
        let mut most_recent_id64: Option<u64> = None;
        let mut in_user_block = false;
        for line in s.lines() {
            let t = line.trim();
            if t.starts_with('"')
                && t.ends_with('"')
                && t.chars().skip(1).take_while(|c| c.is_ascii_digit()).count() + 2 == t.len()
            {
                if let Ok(id64) = t.trim_matches('"').parse::<u64>() {
                    current_id64 = Some(id64);
                }
                in_user_block = true;
                continue;
            }
            if in_user_block && t.contains("\"MostRecent\"") {
                if let Some(val) = Self::extract_value(t) {
                    if val == "1" {
                        most_recent_id64 = current_id64;
                    }
                }
            }
            if in_user_block && t == "}" {
                in_user_block = false;
            }
        }
        let id64 = most_recent_id64.or(current_id64)?;
        let base: u64 = 76561197960265728;
        if id64 > base {
            Some((id64 - base).to_string())
        } else {
            None
        }
    }

    fn get_game_install_dir(&self, app_id: u32) -> Result<PathBuf> {
        for steamapps in self.discover_library_steamapps() {
            let manifest_path = steamapps.join(format!("appmanifest_{}.acf", app_id));
            if manifest_path.exists() {
                let content = fs::read_to_string(&manifest_path)?;
                for line in content.lines() {
                    if line.contains("\"installdir\"") {
                        if let Some(dir) = line.split('"').nth(3) {
                            return Ok(steamapps.join("common").join(dir));
                        }
                    }
                }
            }
        }

        Ok(self
            .steam_path
            .join("steamapps")
            .join("common")
            .join(format!("Game_{}", app_id)))
    }

    pub fn resolve_path(&self, root: u32, filename: &str, app_id: u32) -> Result<PathBuf> {
        let path = match root {
            0 => {
                // Steam默认remote文件夹
                self.steam_path
                    .join("userdata")
                    .join(&self.user_id)
                    .join(app_id.to_string())
                    .join("remote")
                    .join(filename)
            }
            1 => {
                // 游戏安装目录
                self.get_game_install_dir(app_id)?.join(filename)
            }
            2 => {
                // Documents文件夹
                #[cfg(windows)]
                {
                    let docs = std::env::var("USERPROFILE")?;
                    PathBuf::from(docs).join("Documents").join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Documents").join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Documents").join(filename)
                }
            }
            3 => {
                // AppData/Application Support
                #[cfg(windows)]
                {
                    let appdata = std::env::var("APPDATA")?;
                    PathBuf::from(appdata).join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home)
                        .join("Library")
                        .join("Application Support")
                        .join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join(".config").join(filename)
                }
            }
            4 => {
                // LocalAppData/Caches
                #[cfg(windows)]
                {
                    let localappdata = std::env::var("LOCALAPPDATA")?;
                    PathBuf::from(localappdata).join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home)
                        .join("Library")
                        .join("Caches")
                        .join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home)
                        .join(".local")
                        .join("share")
                        .join(filename)
                }
            }
            5 => {
                // Pictures
                #[cfg(windows)]
                {
                    let userprofile = std::env::var("USERPROFILE")?;
                    PathBuf::from(userprofile).join("Pictures").join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Pictures").join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Pictures").join(filename)
                }
            }
            6 => {
                // Music
                #[cfg(windows)]
                {
                    let userprofile = std::env::var("USERPROFILE")?;
                    PathBuf::from(userprofile).join("Music").join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Music").join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Music").join(filename)
                }
            }
            7 => {
                // Videos / Movies
                #[cfg(windows)]
                {
                    let userprofile = std::env::var("USERPROFILE")?;
                    PathBuf::from(userprofile).join("Videos").join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Movies").join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Videos").join(filename)
                }
            }
            8 => {
                // Desktop
                #[cfg(windows)]
                {
                    let userprofile = std::env::var("USERPROFILE")?;
                    PathBuf::from(userprofile).join("Desktop").join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Desktop").join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Desktop").join(filename)
                }
            }
            9 => {
                // Windows Saved Games文件夹 (Vista+)
                #[cfg(windows)]
                {
                    let userprofile = std::env::var("USERPROFILE")?;
                    PathBuf::from(userprofile)
                        .join("Saved Games")
                        .join(filename)
                }
                #[cfg(not(windows))]
                {
                    // macOS/Linux使用Documents/Saved Games
                    let home = std::env::var("HOME")?;
                    #[cfg(target_os = "macos")]
                    let base = PathBuf::from(home).join("Documents").join("Saved Games");
                    #[cfg(target_os = "linux")]
                    let base = PathBuf::from(home).join("Documents").join("Saved Games");
                    base.join(filename)
                }
            }
            10 => {
                // Downloads
                #[cfg(windows)]
                {
                    let userprofile = std::env::var("USERPROFILE")?;
                    PathBuf::from(userprofile).join("Downloads").join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Downloads").join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Downloads").join(filename)
                }
            }
            11 => {
                // Public/Shared
                #[cfg(windows)]
                {
                    PathBuf::from("C:\\Users\\Public").join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    PathBuf::from("/Users/Shared").join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    PathBuf::from("/tmp").join(filename)
                }
            }
            12 => {
                // Windows LocalLow
                #[cfg(windows)]
                {
                    let userprofile = std::env::var("USERPROFILE")?;
                    PathBuf::from(userprofile)
                        .join("AppData")
                        .join("LocalLow")
                        .join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    // 降级到 Caches
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home)
                        .join("Library")
                        .join("Caches")
                        .join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    // 降级到 ~/.local/share
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home)
                        .join(".local")
                        .join("share")
                        .join(filename)
                }
            }
            _ => {
                // 未知root：按照文档降级为 root 0 (remote)
                log::warn!("未知的root值: {}，降级到 remote", root);
                self.steam_path
                    .join("userdata")
                    .join(&self.user_id)
                    .join(app_id.to_string())
                    .join("remote")
                    .join(filename)
            }
        };

        Ok(path)
    }

    // 解析remotecache.vdf文件
    pub fn parse_remotecache(&self, app_id: u32) -> Result<Vec<VdfFileEntry>> {
        let vdf_path = self
            .steam_path
            .join("userdata")
            .join(&self.user_id)
            .join(app_id.to_string())
            .join("remotecache.vdf");

        if !vdf_path.exists() {
            return Err(anyhow!("remotecache.vdf不存在: {:?}", vdf_path));
        }

        log::info!("解析VDF: {:?}", vdf_path);

        let content = fs::read_to_string(&vdf_path)?;
        let mut files = Vec::new();

        let mut pending_key: Option<String> = None;
        let mut in_entry = false;
        let mut current: Option<VdfFileEntry> = None;

        for raw in content.lines() {
            let line = raw.trim();

            if !in_entry {
                if line.starts_with('"') && line.ends_with('"') {
                    let key = line.trim_matches('"');
                    if key.chars().all(|c| c.is_ascii_digit()) {
                        pending_key = None;
                    } else {
                        pending_key = Some(key.to_string());
                    }
                } else if line == "{" {
                    if let Some(name) = pending_key.take() {
                        in_entry = true;
                        current = Some(VdfFileEntry {
                            filename: name,
                            root: 0,
                            size: 0,
                            timestamp: 0,
                            sha: String::new(),
                            sync_state: 0,
                            actual_path: None,
                        });
                    }
                }
                continue;
            }

            if line == "}" {
                if let Some(mut e) = current.take() {
                    if let Ok(path) = self.resolve_path(e.root, &e.filename, app_id) {
                        e.actual_path = Some(path);
                    }
                    files.push(e);
                }
                in_entry = false;
                continue;
            }

            if let Some(e) = current.as_mut() {
                if let Some((key, val)) = Self::extract_key_value(line) {
                    match key {
                        "root" => {
                            e.root = val.parse().unwrap_or(0);
                        }
                        "size" => {
                            e.size = val.parse::<u64>().unwrap_or(0);
                        }
                        "localtime" => {
                            e.timestamp = val.parse::<i64>().unwrap_or(0);
                        }
                        "remotetime" | "time" => {
                            if e.timestamp == 0 {
                                e.timestamp = val.parse::<i64>().unwrap_or(0);
                            }
                        }
                        "sha" => {
                            e.sha = val.to_string();
                        }
                        "syncstate" => {
                            e.sync_state = val.parse().unwrap_or(0);
                        }
                        _ => {}
                    }
                }
            }
        }

        log::info!("解析完成: {} 个文件", files.len());
        Ok(files)
    }

    fn extract_value(line: &str) -> Option<&str> {
        let parts: Vec<&str> = line.split('"').collect();
        if parts.len() >= 4 {
            Some(parts[3])
        } else {
            None
        }
    }

    fn extract_key_value(line: &str) -> Option<(&str, &str)> {
        let mut it = line.split('"');
        it.next()?; // leading content
        let key = it.next()?;
        it.next()?; // separator
        let val = it.next()?;
        Some((key, val))
    }

    pub fn scan_all_cloud_games(&self) -> Result<Vec<CloudGameInfo>> {
        let mut games = Vec::new();
        let userdata_path = self.steam_path.join("userdata").join(&self.user_id);

        log::info!("开始扫描游戏库...");
        let all_manifests = self.scan_app_manifests().unwrap_or_default();
        log::info!("发现 {} 个已安装游戏", all_manifests.len());

        let all_categories = self.parse_shared_config().unwrap_or_default();
        log::info!("解析 {} 个游戏分类", all_categories.len());

        let all_appinfo = self.parse_appinfo_vdf().unwrap_or_default();
        log::info!("从 appinfo.vdf 读取 {} 个游戏信息", all_appinfo.len());

        if let Ok(entries) = fs::read_dir(&userdata_path) {
            for entry in entries.flatten() {
                let entry_name = entry.file_name().to_string_lossy().to_string();
                if let Ok(app_id) = entry_name.parse::<u32>() {
                    let vdf_path = entry.path().join("remotecache.vdf");
                    if vdf_path.exists() {
                        log::debug!("发现云存档游戏: App ID {}", app_id);

                        let files = self.parse_remotecache(app_id).unwrap_or_default();
                        let total_size: u64 = files.iter().map(|f| f.size).sum();
                        let config = self.get_game_config(app_id).ok();
                        let manifest = all_manifests.get(&app_id);
                        let category = all_categories.get(&app_id);
                        let appinfo = all_appinfo.get(&app_id);

                        let game_name = manifest
                            .as_ref()
                            .map(|m| m.name.clone())
                            .or_else(|| appinfo.and_then(|a| a.name.clone()))
                            .or_else(|| Self::fetch_app_name_from_store(app_id));

                        if game_name.is_none() {
                            log::debug!(
                                "App ID {} 无游戏名称 (manifest: {}, appinfo: {})",
                                app_id,
                                manifest.is_some(),
                                appinfo.is_some()
                            );
                        }

                        games.push(CloudGameInfo {
                            app_id,
                            file_count: files.len(),
                            total_size,
                            last_played: config.as_ref().and_then(|c| c.last_played),
                            playtime: config.as_ref().and_then(|c| c.playtime),
                            game_name,
                            is_installed: manifest.is_some(),
                            install_dir: manifest.as_ref().map(|m| m.install_dir.clone()),
                            categories: category
                                .as_ref()
                                .map(|c| c.tags.clone())
                                .unwrap_or_default(),
                        });
                    }
                }
            }
        }

        games.sort_by(|a, b| b.last_played.unwrap_or(0).cmp(&a.last_played.unwrap_or(0)));

        log::info!("扫描完成，共 {} 个有云存档的游戏", games.len());
        Ok(games)
    }
    pub fn get_game_config(&self, app_id: u32) -> Result<GameConfig> {
        let localconfig_path = self
            .steam_path
            .join("userdata")
            .join(&self.user_id)
            .join("config")
            .join("localconfig.vdf");

        if !localconfig_path.exists() {
            return Err(anyhow!("localconfig.vdf不存在"));
        }

        let content = fs::read_to_string(&localconfig_path)?;
        let app_id_str = app_id.to_string();

        let mut last_played = None;
        let mut playtime = None;
        let mut launch_options = None;
        let mut in_app_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.contains(&format!("\"{}\"", app_id_str)) {
                in_app_section = true;
                continue;
            }

            if in_app_section {
                if trimmed == "}" {
                    break;
                }

                if trimmed.contains("\"LastPlayed\"") {
                    if let Some(val) = Self::extract_value(trimmed) {
                        last_played = val.parse().ok();
                    }
                } else if trimmed.contains("\"Playtime\"") {
                    if let Some(val) = Self::extract_value(trimmed) {
                        playtime = val.parse().ok();
                    }
                } else if trimmed.contains("\"LaunchOptions\"") {
                    if let Some(val) = Self::extract_value(trimmed) {
                        launch_options = Some(val.to_string());
                    }
                }
            }
        }

        Ok(GameConfig {
            app_id,
            last_played,
            playtime,
            launch_options,
        })
    }

    pub fn get_all_user_ids(&self) -> Result<Vec<String>> {
        let userdata_path = self.steam_path.join("userdata");
        let mut user_ids = Vec::new();

        if let Ok(entries) = fs::read_dir(&userdata_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.chars().all(|c| c.is_ascii_digit()) {
                    user_ids.push(name);
                }
            }
        }

        Ok(user_ids)
    }

    pub fn with_user_id(steam_path: PathBuf, user_id: String) -> Self {
        Self {
            steam_path,
            user_id,
        }
    }
    pub fn get_steam_path(&self) -> &PathBuf {
        &self.steam_path
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    pub fn scan_app_manifests(&self) -> Result<HashMap<u32, AppManifest>> {
        let mut manifests = HashMap::new();
        for steamapps_path in self.discover_library_steamapps() {
            if let Ok(entries) = fs::read_dir(&steamapps_path) {
                for entry in entries.flatten() {
                    let filename = entry.file_name().to_string_lossy().to_string();
                    if filename.starts_with("appmanifest_") && filename.ends_with(".acf") {
                        if let Ok(manifest) = self.parse_app_manifest(&entry.path()) {
                            manifests.entry(manifest.app_id).or_insert(manifest);
                        }
                    }
                }
            }
        }
        log::info!("扫描到 {} 个已安装游戏", manifests.len());
        Ok(manifests)
    }

    fn discover_library_steamapps(&self) -> Vec<PathBuf> {
        let mut libs = Vec::new();
        let main = self.steam_path.join("steamapps");
        libs.push(main.clone());
        let lf = main.join("libraryfolders.vdf");
        if let Ok(content) = fs::read_to_string(&lf) {
            for line in content.lines() {
                let t = line.trim();
                if t.contains("\"path\"") {
                    if let Some(val) = Self::extract_value(t) {
                        let p = PathBuf::from(val).join("steamapps");
                        if p.exists() {
                            libs.push(p);
                        }
                    }
                }
            }
        }
        libs
    }

    fn parse_app_manifest(&self, path: &Path) -> Result<AppManifest> {
        let content = fs::read_to_string(path)?;
        let mut app_id = None;
        let mut name = None;
        let mut install_dir = None;
        let mut size_on_disk = None;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.contains("\"appid\"") {
                if let Some(val) = Self::extract_value(trimmed) {
                    app_id = val.parse().ok();
                }
            } else if trimmed.contains("\"name\"") {
                if let Some(val) = Self::extract_value(trimmed) {
                    name = Some(val.to_string());
                }
            } else if trimmed.contains("\"installdir\"") {
                if let Some(val) = Self::extract_value(trimmed) {
                    install_dir = Some(val.to_string());
                }
            } else if trimmed.contains("\"SizeOnDisk\"") {
                if let Some(val) = Self::extract_value(trimmed) {
                    size_on_disk = val.parse().ok();
                }
            }
        }

        if let (Some(app_id), Some(name), Some(install_dir)) = (app_id, name, install_dir) {
            Ok(AppManifest {
                app_id,
                name,
                install_dir,
                size_on_disk,
            })
        } else {
            Err(anyhow!("解析 manifest 失败"))
        }
    }

    pub fn parse_shared_config(&self) -> Result<HashMap<u32, GameCategory>> {
        let mut categories = HashMap::new();
        let sharedconfig_path = self
            .steam_path
            .join("userdata")
            .join(&self.user_id)
            .join("7")
            .join("remote")
            .join("sharedconfig.vdf");

        if !sharedconfig_path.exists() {
            return Ok(categories);
        }

        let content = fs::read_to_string(&sharedconfig_path)?;
        let mut current_app_id: Option<u32> = None;
        let mut current_tags = Vec::new();
        let mut is_favorite = false;
        let mut is_hidden = false;
        let mut in_tags_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('"') && trimmed.ends_with('"') {
                if let Ok(app_id) = trimmed.trim_matches('"').parse::<u32>() {
                    if let Some(prev_app_id) = current_app_id {
                        categories.insert(
                            prev_app_id,
                            GameCategory {
                                app_id: prev_app_id,
                                tags: current_tags.clone(),
                                is_favorite,
                                is_hidden,
                            },
                        );
                    }
                    current_app_id = Some(app_id);
                    current_tags.clear();
                    is_favorite = false;
                    is_hidden = false;
                    in_tags_section = false;
                }
            }

            if trimmed.contains("\"tags\"") {
                in_tags_section = true;
            }

            if in_tags_section && trimmed.starts_with('"') && trimmed.contains('"') {
                if let Some(tag) = Self::extract_value(trimmed) {
                    if !tag.is_empty() && tag != "tags" {
                        current_tags.push(tag.to_string());
                    }
                }
            }

            if trimmed.contains("\"favorite\"") {
                is_favorite = true;
            }

            if trimmed.contains("\"hidden\"") {
                is_hidden = true;
            }

            if in_tags_section && trimmed == "}" {
                in_tags_section = false;
            }
        }

        if let Some(prev_app_id) = current_app_id {
            categories.insert(
                prev_app_id,
                GameCategory {
                    app_id: prev_app_id,
                    tags: current_tags,
                    is_favorite,
                    is_hidden,
                },
            );
        }

        log::info!("解析到 {} 个游戏的分类信息", categories.len());
        Ok(categories)
    }

    #[allow(dead_code)]
    pub fn get_installed_games(&self) -> Result<Vec<AppManifest>> {
        let manifests = self.scan_app_manifests()?;
        Ok(manifests.into_values().collect())
    }

    #[allow(dead_code)]
    pub fn is_game_installed(&self, app_id: u32) -> bool {
        let manifest_path = self
            .steam_path
            .join("steamapps")
            .join(format!("appmanifest_{}.acf", app_id));
        manifest_path.exists()
    }

    pub fn parse_appinfo_vdf(&self) -> Result<HashMap<u32, AppInfo>> {
        let appinfo_path = self.steam_path.join("appcache").join("appinfo.vdf");

        if !appinfo_path.exists() {
            log::debug!("appinfo.vdf 不存在，跳过解析");
            return Ok(HashMap::new());
        }

        let data = match fs::read(&appinfo_path) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("无法读取 appinfo.vdf: {}", e);
                return Ok(HashMap::new());
            }
        };

        let mut cursor = Cursor::new(data);
        let mut apps = HashMap::new();

        let magic = match cursor.read_u32::<LittleEndian>() {
            Ok(m) => m,
            Err(_) => {
                log::warn!("appinfo.vdf 格式无效");
                return Ok(HashMap::new());
            }
        };

        if magic != 0x07564427 && magic != 0x07564428 && magic != 0x07564429 {
            log::warn!("appinfo.vdf 格式不支持: 0x{:X}", magic);
            return Ok(HashMap::new());
        }

        let _ = cursor.read_u32::<LittleEndian>();

        let mut count = 0;
        while let Ok(app_id) = cursor.read_u32::<LittleEndian>() {
            if app_id == 0 || count > 10000 {
                break;
            }

            // Skip size, infostate, last_updated, access_token
            for _ in 0..3 {
                let _ = cursor.read_u32::<LittleEndian>();
            }
            let _ = cursor.read_u64::<LittleEndian>();

            // Read SHA hash (20 bytes)
            let mut sha = vec![0u8; 20];
            if cursor.read_exact(&mut sha).is_err() {
                break;
            }

            // Skip change_number (4 bytes)
            let _ = cursor.read_u32::<LittleEndian>();

            // Try to find the game name in the VDF structure
            if let Ok(name) = Self::parse_appinfo_name(&mut cursor, app_id) {
                if !name.is_empty() && name.len() < 200 {
                    apps.insert(
                        app_id,
                        AppInfo {
                            app_id,
                            name: Some(name),
                            developer: None,
                            publisher: None,
                        },
                    );
                }
            }

            // Skip to next entry (read remaining data)
            let mut buf = vec![0u8; 4096];
            let mut skipped = 0;
            while skipped < 500000 {
                if cursor.read(&mut buf).is_err() {
                    break;
                }
                skipped += buf.len();
                // Look for next app_id marker or end
                if buf.starts_with(&[0, 0, 0, 0]) {
                    break;
                }
            }

            count += 1;
        }

        log::info!("从 appinfo.vdf 解析到 {} 个游戏", apps.len());
        Ok(apps)
    }

    fn fetch_app_name_from_store(app_id: u32) -> Option<String> {
        let url = format!(
            "https://store.steampowered.com/api/appdetails?appids={}&l=schinese",
            app_id
        );
        let resp = ureq::get(&url).call().ok()?;
        let text = resp.into_string().ok()?;
        let v: serde_json::Value = serde_json::from_str(&text).ok()?;
        let key = app_id.to_string();
        let data = v.get(&key)?.get("data")?;
        data.get("name")?.as_str().map(|s: &str| s.to_string())
    }

    fn parse_appinfo_name(cursor: &mut Cursor<Vec<u8>>, app_id: u32) -> Result<String> {
        // VDF binary format: try to find "name" field
        let mut buf = vec![0u8; 1024];
        if cursor.read(&mut buf).is_err() {
            return Err(anyhow!("无法读取"));
        }

        // Look for "common" section and "name" field
        let buf_str = String::from_utf8_lossy(&buf);

        // Try to find name pattern
        if let Some(name_pos) = buf_str.find("name\0") {
            let start = name_pos + 5; // Skip "name\0"
            if start < buf.len() {
                // Find the string after "name"
                let remaining = &buf[start..];
                if let Some(null_pos) = remaining.iter().position(|&b| b == 0) {
                    if let Ok(name) = String::from_utf8(remaining[..null_pos].to_vec()) {
                        if !name.is_empty() && name.is_ascii() {
                            log::debug!("App {} 名称: {}", app_id, name);
                            return Ok(name);
                        }
                    }
                }
            }
        }

        Err(anyhow!("未找到游戏名称"))
    }

    #[allow(dead_code)]
    fn read_simple_string(cursor: &mut Cursor<Vec<u8>>, max_len: usize) -> Result<String> {
        let mut bytes = Vec::new();
        for _ in 0..max_len {
            match cursor.read_u8() {
                Ok(0) => break,
                Ok(b) if b < 128 && (b.is_ascii_graphic() || b == b' ') => bytes.push(b),
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        if bytes.is_empty() {
            return Err(anyhow!("空字符串"));
        }
        String::from_utf8(bytes).map_err(|e| anyhow!("UTF-8 解码失败: {}", e))
    }
}
