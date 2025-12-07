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
    // 尝试解析 Root 类型
    let root_type = match RootType::from_u32(root) {
        Some(rt) => rt,
        None => {
            log::debug!("未知的 root 值: {}，使用 SteamRemote", root);
            RootType::SteamRemote
        }
    };

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

// 收集本地存档路径
pub fn collect_local_save_paths(
    files: &[crate::steam_api::CloudFile],
    steam_path: &Path,
    user_id: &str,
    app_id: u32,
) -> Vec<(String, PathBuf)> {
    use std::collections::HashMap;

    tracing::debug!(
        "开始收集本地存档路径: app_id={}, 文件数={}",
        app_id,
        files.len()
    );

    // 按父目录去重，而不是按 root 类型
    let mut path_map: HashMap<PathBuf, (String, PathBuf)> = HashMap::new();
    let mut failed_count = 0;

    for file in files {
        // 解析完整文件路径
        let file_path_result =
            resolve_cloud_file_path(file.root, &file.name, steam_path, user_id, app_id);

        match file_path_result {
            Ok(file_path) => {
                // 获取父目录（存档文件夹）
                if let Some(parent) = file_path.parent() {
                    let parent_path = parent.to_path_buf();

                    // 检查父目录是否存在
                    if parent_path.exists() {
                        // 如果这个父目录还没有记录，添加它
                        path_map.entry(parent_path.clone()).or_insert_with(|| {
                            let desc = get_root_description(file.root);
                            tracing::debug!("✓ {}: {}", desc, parent_path.display());
                            (desc, parent_path.clone())
                        });
                    } else {
                        tracing::trace!(
                            "父目录不存在: root={}, file={}, path={}",
                            file.root,
                            file.name,
                            parent_path.display()
                        );
                    }
                }
            }
            Err(e) => {
                failed_count += 1;
                tracing::trace!(
                    "解析路径失败: root={}, file={}, error={}",
                    file.root,
                    file.name,
                    e
                );
            }
        }
    }

    // 记录失败统计
    if failed_count > 0 {
        tracing::debug!("有 {} 个文件路径解析失败", failed_count);
    }

    let mut paths: Vec<(String, PathBuf)> = path_map.into_values().collect();
    paths.sort_by(|a, b| a.0.cmp(&b.0));

    if !paths.is_empty() {
        tracing::info!("检测到 {} 个本地存档根目录", paths.len());
        for (desc, path) in &paths {
            tracing::info!("  ✓ {}: {}", desc, path.display());
        }
    } else {
        tracing::warn!(
            "未找到任何本地存档路径 (app_id={})，请检查游戏是否已安装",
            app_id
        );
    }

    paths
}

// 获取 Root 类型的描述文本，格式：CDP文件夹名 (Root编号)
pub fn get_root_description(root: u32) -> String {
    let cdp_name = get_cdp_folder_name(root);
    format!("{} ({})", cdp_name, root)
}

// 获取 CDP 文件夹名称
pub fn get_cdp_folder_name(root: u32) -> &'static str {
    match root {
        0 => "Steam Cloud",
        1 => "GameInstall",
        2 => "Documents",
        3 => "AppData Roaming",
        4 => "AppData Local",
        5 => "Pictures",
        6 => "Music",
        7 => {
            #[cfg(target_os = "macos")]
            return "MacAppSupport";
            #[cfg(not(target_os = "macos"))]
            return "Videos";
        }
        8 => "Desktop",
        9 => "Saved Games",
        10 => "Downloads",
        11 => "Public",
        12 => "AppData LocalLow",
        _ => "Unknown",
    }
}

// CDP 网页上的 folder 名称需要映射到标准描述
pub fn normalize_cdp_folder_name(folder: &str) -> String {
    match folder.to_lowercase().as_str() {
        "steam cloud" | "steamcloud" | "" => get_root_description(0),
        "gameinstall" | "game install" => get_root_description(1),
        "documents" | "my documents" => get_root_description(2),
        "appdata roaming" | "roaming" => get_root_description(3),
        "appdata local" | "local" => get_root_description(4),
        "pictures" => get_root_description(5),
        "music" => get_root_description(6),
        "videos" | "movies" | "macappsupport" => get_root_description(7),
        "desktop" => get_root_description(8),
        "saved games" => get_root_description(9),
        "downloads" => get_root_description(10),
        "public" | "shared" => get_root_description(11),
        "appdata locallow" | "locallow" => get_root_description(12),
        _ => folder.to_string(),
    }
}
