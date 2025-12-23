// 路径解析模块

// Steam Cloud 存储位置类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RootType {
    // 0: Steam 云文件夹 (userdata/{ID}/{AppID}/remote/)
    SteamRemote = 0,
    // 1: 游戏安装目录
    GameInstallDir = 1,
    // 2: 文档文件夹
    Documents = 2,
    // 3: AppData Roaming (Win) / Application Support (Mac) / .config (Linux)
    AppDataRoaming = 3,
    // 4: Local AppData (Win) / Caches (Mac) / .local/share (Linux)
    AppDataLocal = 4,
    // 5: 图片文件夹
    Pictures = 5,
    // 6: 音乐文件夹
    Music = 6,
    // 7: 视频文件夹 (Win/Linux: Videos, Mac: Application Support)
    Videos = 7,
    // 8: 桌面文件夹
    Desktop = 8,
    // 9: Windows Saved Games
    SavedGames = 9,
    // 10: 下载文件夹
    Downloads = 10,
    // 11: 公共共享目录
    PublicShared = 11,
    // 12: Windows LocalLow / Caches (Mac/Linux)
    AppDataLocalLow = 12,
}

impl RootType {
    // 从 u32 转换为 RootType
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::SteamRemote),
            1 => Some(Self::GameInstallDir),
            2 => Some(Self::Documents),
            3 => Some(Self::AppDataRoaming),
            4 => Some(Self::AppDataLocal),
            5 => Some(Self::Pictures),
            6 => Some(Self::Music),
            7 => Some(Self::Videos),
            8 => Some(Self::Desktop),
            9 => Some(Self::SavedGames),
            10 => Some(Self::Downloads),
            11 => Some(Self::PublicShared),
            12 => Some(Self::AppDataLocalLow),
            _ => None,
        }
    }

    // 转换为 u32
    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

// 解析 Root 类型的基础路径
pub fn resolve_root_base_path(
    root_type: RootType,
    steam_path: &Path,
    user_id: &str,
    app_id: u32,
) -> Result<PathBuf> {
    match root_type {
        RootType::SteamRemote => Ok(steam_path
            .join("userdata")
            .join(user_id)
            .join(app_id.to_string())
            .join("remote")),

        RootType::GameInstallDir => Err(anyhow!("GameInstallDir 需要特殊处理")),

        RootType::Documents => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("Documents"))
            }
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Documents"))
            }
            #[cfg(target_os = "linux")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Documents"))
            }
        }

        RootType::AppDataRoaming => {
            #[cfg(target_os = "windows")]
            {
                let appdata = std::env::var("APPDATA")?;
                Ok(PathBuf::from(appdata))
            }
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home)
                    .join("Library")
                    .join("Application Support"))
            }
            #[cfg(target_os = "linux")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join(".config"))
            }
        }

        RootType::AppDataLocal => {
            #[cfg(target_os = "windows")]
            {
                let localappdata = std::env::var("LOCALAPPDATA")?;
                Ok(PathBuf::from(localappdata))
            }
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Library").join("Caches"))
            }
            #[cfg(target_os = "linux")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join(".local").join("share"))
            }
        }

        RootType::Pictures => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("Pictures"))
            }
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Pictures"))
            }
            #[cfg(target_os = "linux")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Pictures"))
            }
        }

        RootType::Music => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("Music"))
            }
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Music"))
            }
            #[cfg(target_os = "linux")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Music"))
            }
        }

        RootType::Videos => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("Videos"))
            }
            #[cfg(target_os = "macos")]
            {
                // Root=7 在 macOS 上实际映射到 Application Support，而不是 Movies
                // 参考: Finding Paradise 游戏实际使用情况
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home)
                    .join("Library")
                    .join("Application Support"))
            }
            #[cfg(target_os = "linux")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Videos"))
            }
        }

        RootType::Desktop => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("Desktop"))
            }
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Desktop"))
            }
            #[cfg(target_os = "linux")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Desktop"))
            }
        }

        RootType::SavedGames => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("Saved Games"))
            }
            #[cfg(not(target_os = "windows"))]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Documents").join("Saved Games"))
            }
        }

        RootType::Downloads => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("Downloads"))
            }
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Downloads"))
            }
            #[cfg(target_os = "linux")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Downloads"))
            }
        }

        RootType::PublicShared => {
            #[cfg(target_os = "windows")]
            {
                Ok(PathBuf::from("C:/Users/Public"))
            }
            #[cfg(target_os = "macos")]
            {
                Ok(PathBuf::from("/Users/Shared"))
            }
            #[cfg(target_os = "linux")]
            {
                Ok(PathBuf::from("/tmp"))
            }
        }

        RootType::AppDataLocalLow => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("AppData").join("LocalLow"))
            }
            #[cfg(not(target_os = "windows"))]
            {
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    Ok(PathBuf::from(home).join("Library").join("Caches"))
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    Ok(PathBuf::from(home).join(".local").join("share"))
                }
            }
        }
    }
}

// 解析云文件的完整路径
pub fn resolve_cloud_file_path(
    root: u32,
    filename: &str,
    steam_path: &Path,
    user_id: &str,
    app_id: u32,
) -> Result<PathBuf> {
    let root_type = RootType::from_u32(root).unwrap_or_else(|| {
        log::debug!("未知的 root 值: {}，使用 SteamRemote", root);
        RootType::SteamRemote
    });

    // GameInstallDir 需要查找游戏安装目录
    if root_type == RootType::GameInstallDir {
        let install_dir = get_game_install_dir(steam_path, app_id)?;
        return Ok(install_dir.join(filename));
    }

    let base_path = resolve_root_base_path(root_type, steam_path, user_id, app_id)?;
    Ok(base_path.join(filename))
}

// 获取游戏安装目录
// - Windows/Linux: 游戏安装目录
// - macOS: ~/Library/Application Support/{GameName}/
fn get_game_install_dir(steam_path: &Path, app_id: u32) -> Result<PathBuf> {
    let libraries = crate::game_scanner::discover_library_steamapps(steam_path);

    for steamapps in libraries.iter() {
        let manifest_path = steamapps.join(format!("appmanifest_{}.acf", app_id));

        if manifest_path.exists() {
            match std::fs::read_to_string(&manifest_path) {
                Ok(content) => {
                    let mut install_dir: Option<String> = None;
                    #[cfg(target_os = "macos")]
                    let mut name: Option<String> = None;

                    for line in content.lines() {
                        if line.contains("\"installdir\"") {
                            if let Some(dir) = line.split('"').nth(3) {
                                install_dir = Some(dir.to_string());
                            }
                        }
                        #[cfg(target_os = "macos")]
                        if line.contains("\"name\"") {
                            if let Some(n) = line.split('"').nth(3) {
                                name = Some(n.to_string());
                            }
                        }
                    }

                    if let Some(dir) = install_dir {
                        #[cfg(target_os = "macos")]
                        {
                            if let Some(ref gname) = name {
                                let home = std::env::var("HOME")?;
                                let app_support_path = std::path::PathBuf::from(home)
                                    .join("Library")
                                    .join("Application Support")
                                    .join(gname);

                                if app_support_path.exists() {
                                    tracing::info!(
                                        "找到 macOS 存档目录: {}",
                                        app_support_path.display()
                                    );
                                    return Ok(app_support_path);
                                }
                            }
                        }

                        // 尝试游戏安装目录
                        let install_path = steamapps.join("common").join(&dir);
                        if install_path.exists() {
                            tracing::info!("找到游戏安装目录: {}", install_path.display());
                            return Ok(install_path);
                        } else {
                            // 继续返回路径，即使目录不存在
                            return Ok(install_path);
                        }
                    }
                }
                Err(_) => {
                    // 静默失败，继续查找
                }
            }
        }
    }

    // 如果找不到，返回错误而不是默认路径
    let error_msg = format!("未找到游戏 {} 的安装目录，请确认游戏已安装", app_id);
    tracing::warn!("{}", error_msg);
    Err(anyhow!(error_msg))
}

// 收集本地存档路径（使用根基础路径）
/// 基于 appinfo.vdf savefiles 配置收集本地存档路径
///
/// 逻辑：
/// 1. 默认添加 root=0 (SteamRemote) 目录
/// 2. 根据 savefiles 配置添加其他 root 类型目录
/// 3. 过滤平台不匹配的配置
pub fn collect_local_save_paths_from_ufs(
    savefiles: &[SaveFileConfig],
    steam_path: &Path,
    user_id: &str,
    app_id: u32,
) -> Vec<(String, PathBuf)> {
    use std::collections::HashMap;

    tracing::debug!(
        "开始收集本地存档路径 (基于 appinfo.vdf): app_id={}, savefiles配置数={}",
        app_id,
        savefiles.len()
    );

    let mut path_map: HashMap<u32, (String, PathBuf)> = HashMap::new();

    // 1. 默认添加 root=0 (SteamRemote) 目录
    let remote_path = steam_path
        .join("userdata")
        .join(user_id)
        .join(app_id.to_string())
        .join("remote");

    if remote_path.exists() {
        let desc = get_root_description(0);
        tracing::debug!("✓ {} (默认): {}", desc, remote_path.display());
        path_map.insert(0, (desc, remote_path));
    }

    // 2. 根据 savefiles 配置添加其他 root 类型目录
    // 预先缓存 game_install_dir，避免重复查找
    let game_install_dir_cache = get_game_install_dir(steam_path, app_id).ok();

    for config in savefiles {
        // 检查平台是否匹配
        if !platform_matches_current(&config.platforms) {
            tracing::debug!(
                "跳过不匹配平台的配置: root={}, platforms={:?}",
                config.root,
                config.platforms
            );
            continue;
        }

        // 获取 root 类型
        let root_type = match &config.root_type {
            Some(rt) => *rt,
            None => {
                tracing::debug!("无法解析 root 类型: {}", config.root);
                continue;
            }
        };

        let root_num = root_type.to_u32();

        // 跳过已经处理过的 root 类型
        if path_map.contains_key(&root_num) {
            continue;
        }

        // 解析基础路径
        let base_path = if root_num == 1 {
            // GameInstallDir 使用缓存
            game_install_dir_cache.clone()
        } else {
            resolve_root_base_path(root_type, steam_path, user_id, app_id).ok()
        };

        if let Some(base_path) = base_path {
            if base_path.exists() {
                let desc = get_root_description(root_num);
                tracing::debug!("✓ {} (appinfo.vdf): {}", desc, base_path.display());
                path_map.insert(root_num, (desc, base_path));
            }
        }
    }

    let paths: Vec<(String, PathBuf)> = path_map.into_values().collect();

    if !paths.is_empty() {
        tracing::info!("检测到 {} 个本地存档根目录", paths.len());
        for (desc, path) in &paths {
            tracing::info!("  ✓ {}: {}", desc, path.display());
        }
    } else {
        tracing::warn!("未找到任何本地存档路径 (app_id={})", app_id);
    }

    paths
}

// 获取 Root 类型的描述文本，格式：Root类型名 (Root编号)
pub fn get_root_description(root: u32) -> String {
    let type_name = get_root_type_name(root);
    format!("{} ({})", type_name, root)
}

/// 解析 CDP 格式的 root_description
/// 格式: "CDP:<url>|<folder>"
/// 返回: (url, folder)
pub fn parse_cdp_root_description(root_description: &str) -> (Option<&str>, &str) {
    if let Some(content) = root_description.strip_prefix("CDP:") {
        let mut parts = content.split('|');
        let url = parts.next().filter(|s| !s.is_empty());
        let folder = parts.next().unwrap_or(root_description);
        (url, folder)
    } else {
        (None, root_description)
    }
}

// 获取 Root 类型名称
pub fn get_root_type_name(root: u32) -> &'static str {
    match root {
        0 => "SteamRemote",
        1 => "GameInstall",
        2 => "Documents",
        3 => "AppDataRoaming",
        4 => "AppDataLocal",
        5 => "Pictures",
        6 => "Music",
        7 => "Videos",
        8 => "Desktop",
        9 => "SavedGames",
        10 => "Downloads",
        11 => "Public",
        12 => "AppDataLocalLow",
        _ => "Unknown",
    }
}

// 从 appinfo.vdf 的 root 字符串名称转换为 RootType
pub fn root_name_to_type(name: &str) -> Option<RootType> {
    match name {
        "SteamCloudDocuments" | "0" => Some(RootType::SteamRemote),
        "GameInstall" | "1" => Some(RootType::GameInstallDir),
        "WinMyDocuments" | "2" => Some(RootType::Documents),
        "WinAppDataRoaming" | "3" => Some(RootType::AppDataRoaming),
        "WinAppDataLocal" | "4" => Some(RootType::AppDataLocal),
        "WinPictures" | "5" => Some(RootType::Pictures),
        "WinMusic" | "6" => Some(RootType::Music),
        "WinVideos" | "MacAppSupport" | "7" => Some(RootType::Videos),
        "LinuxXdgDataHome" | "8" => Some(RootType::Desktop),
        "WinSavedGames" | "9" => Some(RootType::SavedGames),
        "WinDownloads" | "10" => Some(RootType::Downloads),
        "WinPublic" | "11" => Some(RootType::PublicShared),
        "WinAppDataLocalLow" | "12" => Some(RootType::AppDataLocalLow),
        _ => None,
    }
}

// 检查平台是否匹配当前系统
pub fn platform_matches_current(platforms: &[String]) -> bool {
    if platforms.is_empty() {
        return true; // 没有平台限制 = 所有平台
    }

    let current_platform = get_current_platform();
    platforms.iter().any(|p| {
        let p_lower = p.to_lowercase();
        match current_platform {
            "windows" => p_lower.contains("windows") || p_lower.contains("win"),
            "macos" => {
                p_lower.contains("macos") || p_lower.contains("mac") || p_lower.contains("osx")
            }
            "linux" => p_lower.contains("linux"),
            _ => false,
        }
    })
}

// 获取当前平台名称
pub fn get_current_platform() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
}

// 从 ufs savefiles 配置中的路径配置
#[derive(Debug, Clone, Default)]
pub struct SaveFileConfig {
    pub root: String,                // root 字符串名称 (如 "WinMyDocuments")
    pub root_type: Option<RootType>, // 解析后的 RootType
    pub path: String,                // 子目录路径
    pub pattern: String,             // 文件匹配模式 (glob)
    pub platforms: Vec<String>,      // 支持的平台
    pub recursive: bool,             // 是否递归 (默认 true)
}

// 扫描到的本地文件信息
#[derive(Debug, Clone)]
pub struct ScannedLocalFile {
    pub relative_path: String, // 相对于 root 的路径 (用于与云端文件名匹配)
    pub size: u64,
    pub modified: std::time::SystemTime,
}

// 根据 ufs savefiles 配置扫描本地文件
pub fn scan_local_files_from_ufs(
    savefiles: &[SaveFileConfig],
    steam_path: &Path,
    user_id: &str,
    app_id: u32,
) -> Vec<ScannedLocalFile> {
    let mut results = Vec::new();

    for config in savefiles {
        // 检查平台是否匹配
        if !platform_matches_current(&config.platforms) {
            tracing::debug!(
                "跳过不匹配平台的配置: root={}, platforms={:?}",
                config.root,
                config.platforms
            );
            continue;
        }

        // 获取 root 类型
        let root_type = match config.root_type {
            Some(rt) => rt,
            None => {
                tracing::warn!("无法解析 root 类型: {}", config.root);
                continue;
            }
        };

        // 解析基础路径
        let base_path = match resolve_root_base_path(root_type, steam_path, user_id, app_id) {
            Ok(p) => p,
            Err(e) => {
                tracing::debug!("无法解析 root 路径: {} - {}", config.root, e);
                continue;
            }
        };

        // 构建完整扫描路径
        let scan_path = if config.path.is_empty() {
            base_path.clone()
        } else {
            // 移除路径开头的斜杠
            let clean_path = config.path.trim_start_matches('/').trim_start_matches('\\');
            base_path.join(clean_path)
        };

        if !scan_path.exists() {
            tracing::debug!("扫描目录不存在: {}", scan_path.display());
            continue;
        }

        tracing::debug!(
            "扫描本地目录: {} (pattern={})",
            scan_path.display(),
            config.pattern
        );

        // 扫描目录
        let files = scan_directory_with_pattern(&scan_path, &config.pattern, config.recursive);

        for (full_path, relative_to_scan) in files {
            // 计算相对于 base_path 的路径 (用于匹配云端文件名)
            let relative_path = if config.path.is_empty() {
                relative_to_scan
            } else {
                let clean_path = config.path.trim_start_matches('/').trim_start_matches('\\');
                format!("{}/{}", clean_path, relative_to_scan)
            };

            if let Ok(metadata) = std::fs::metadata(&full_path) {
                results.push(ScannedLocalFile {
                    relative_path,
                    size: metadata.len(),
                    modified: metadata
                        .modified()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                });
            }
        }
    }

    tracing::info!("从 ufs 配置扫描到 {} 个本地文件", results.len());
    results
}

// 根据 pattern 扫描目录
fn scan_directory_with_pattern(
    dir: &Path,
    pattern: &str,
    recursive: bool,
) -> Vec<(PathBuf, String)> {
    let mut results = Vec::new();

    fn scan_dir(
        dir: &Path,
        base: &Path,
        pattern: &str,
        recursive: bool,
        results: &mut Vec<(PathBuf, String)>,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() && recursive {
                scan_dir(&path, base, pattern, recursive, results);
            } else if path.is_file() {
                // 检查文件名是否匹配 pattern
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if pattern_matches(filename, pattern) {
                        // 计算相对路径
                        if let Ok(rel) = path.strip_prefix(base) {
                            let rel_str = rel.to_string_lossy().replace('\\', "/");
                            results.push((path.clone(), rel_str));
                        }
                    }
                }
            }
        }
    }

    scan_dir(dir, dir, pattern, recursive, &mut results);
    results
}

// 简单的 glob pattern 匹配
fn pattern_matches(filename: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern == "*.*" {
        return true;
    }

    // 简单的 *.ext 匹配
    if let Some(ext) = pattern.strip_prefix("*.") {
        return filename.ends_with(&format!(".{}", ext));
    }

    // 简单的前缀* 匹配
    if let Some(prefix) = pattern.strip_suffix('*') {
        return filename.starts_with(prefix);
    }

    // 精确匹配
    filename == pattern
}
