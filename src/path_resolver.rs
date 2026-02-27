// 路径解析模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

// 游戏安装目录缓存，避免重复解析
static GAME_INSTALL_DIR_CACHE: std::sync::LazyLock<Mutex<HashMap<u32, PathBuf>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

// Root Overrides 缓存，存储每个 app 的 rootoverrides 配置
static ROOT_OVERRIDES_CACHE: std::sync::LazyLock<Mutex<HashMap<u32, Vec<RootOverrideConfig>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

// SteamCloudDocuments 路径缓存
static STEAM_CLOUD_DOCS_CACHE: std::sync::LazyLock<Mutex<HashMap<u32, PathBuf>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

// 游戏名称缓存，从 game_scanner/CDP 获取
static GAME_NAME_CACHE: std::sync::LazyLock<Mutex<HashMap<u32, String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn set_game_name_cache(app_id: u32, name: String) {
    if let Ok(mut cache) = GAME_NAME_CACHE.lock() {
        cache.insert(app_id, name);
    }
}

fn get_game_name_cached(app_id: u32) -> Option<String> {
    if let Ok(cache) = GAME_NAME_CACHE.lock() {
        return cache.get(&app_id).cloned();
    }
    None
}

// 设置某个 app 的 rootoverrides 缓存
pub fn set_root_overrides_cache(app_id: u32, overrides: Vec<RootOverrideConfig>) {
    if let Ok(mut cache) = ROOT_OVERRIDES_CACHE.lock() {
        cache.insert(app_id, overrides);
    }
}

// 获取某个 app 的 rootoverrides 缓存
fn get_root_overrides_cache(app_id: u32) -> Option<Vec<RootOverrideConfig>> {
    if let Ok(cache) = ROOT_OVERRIDES_CACHE.lock() {
        cache.get(&app_id).cloned()
    } else {
        None
    }
}

// Steam Cloud 存储位置类型
// 字符串名称来源: https://partner.steamgames.com/doc/features/cloud
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum RootType {
    // Root 0: API 默认路径
    // 路径: {Steam}/userdata/{UID}/{AppID}/remote/
    // 通过 ISteamRemoteStorage API (FileWrite/FileRead) 操作的文件存储在此
    DefaultApi = 0,

    // Root 1: GameInstall - 游戏安装目录
    // 路径: {Steam}/steamapps/common/{Game}/
    GameInstall = 1,

    // Root 2: WinMyDocuments
    // Win: %USERPROFILE%\Documents, Mac/Linux: ~/Documents
    WinMyDocuments = 2,

    // Root 3: WinAppDataLocal
    // Win: %LOCALAPPDATA%, Mac: ~/, Linux: ~/.local/share
    WinAppDataLocal = 3,

    // Root 4: WinAppDataRoaming
    // Win: %APPDATA%, Mac: ~/Library/Application Support, Linux: ~/.config
    WinAppDataRoaming = 4,

    // Root 5: SteamUserBaseStorage
    SteamUserBaseStorage = 5,

    // Root 6: MacHome
    // Mac: ~/, Win: %USERPROFILE%, Linux: ~/
    MacHome = 6,

    // Root 7: MacAppSupport
    // Mac: ~/Library/Application Support, Win: %APPDATA%, Linux: ~/.config
    MacAppSupport = 7,

    // Root 8: MacDocuments
    // Mac/Linux: ~/Documents, Win: %USERPROFILE%\Documents
    MacDocuments = 8,

    // Root 9: WinSavedGames
    // Win: %USERPROFILE%\Saved Games
    WinSavedGames = 9,

    // Root 10: WinProgramData
    // Win: %PROGRAMDATA%
    WinProgramData = 10,

    // Root 11: SteamCloudDocuments (UFS Auto-Cloud)
    // Mac: ~/Documents/Steam Cloud/[用户名]/[游戏名]/
    // Linux: ~/.SteamCloud/[用户名]/[游戏名]/
    // Win: %USERPROFILE%\Documents\Steam Cloud\[用户名]\[游戏名]\ (推测)
    SteamCloudDocuments = 11,

    // Root 12: WinAppDataLocalLow
    // Win: %USERPROFILE%\AppData\LocalLow
    WinAppDataLocalLow = 12,

    // Root 13: MacCaches
    // Mac: ~/Library/Caches
    MacCaches = 13,

    // Root 14: LinuxHome
    // Linux: ~/, Mac: ~/, Win: %USERPROFILE%
    LinuxHome = 14,

    // Root 15: LinuxXdgDataHome
    // Linux: $XDG_DATA_HOME 或 ~/.local/share
    LinuxXdgDataHome = 15,

    // Root 16: LinuxXdgConfigHome
    // Linux: $XDG_CONFIG_HOME 或 ~/.config
    LinuxXdgConfigHome = 16,

    // Root 17: AndroidSteamPackageRoot
    AndroidSteamPackageRoot = 17,

    // Root 18: WindowsHome
    // Win: %USERPROFILE%, Mac: ~/, Linux: ~/
    WindowsHome = 18,
}

impl RootType {
    // 从 u32 转换为 RootType
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::DefaultApi),
            1 => Some(Self::GameInstall),
            2 => Some(Self::WinMyDocuments),
            3 => Some(Self::WinAppDataLocal),
            4 => Some(Self::WinAppDataRoaming),
            5 => Some(Self::SteamUserBaseStorage),
            6 => Some(Self::MacHome),
            7 => Some(Self::MacAppSupport),
            8 => Some(Self::MacDocuments),
            9 => Some(Self::WinSavedGames),
            10 => Some(Self::WinProgramData),
            11 => Some(Self::SteamCloudDocuments),
            12 => Some(Self::WinAppDataLocalLow),
            13 => Some(Self::MacCaches),
            14 => Some(Self::LinuxHome),
            15 => Some(Self::LinuxXdgDataHome),
            16 => Some(Self::LinuxXdgConfigHome),
            17 => Some(Self::AndroidSteamPackageRoot),
            18 => Some(Self::WindowsHome),
            _ => None,
        }
    }

    // 转换为 u32
    pub fn to_u32(self) -> u32 {
        self as u32
    }

    // 转换为名称字符串
    pub fn to_name(self) -> &'static str {
        match self {
            Self::DefaultApi => "Default(API)",
            Self::GameInstall => "GameInstall",
            Self::WinMyDocuments => "WinMyDocuments",
            Self::WinAppDataLocal => "WinAppDataLocal",
            Self::WinAppDataRoaming => "WinAppDataRoaming",
            Self::SteamUserBaseStorage => "SteamUserBaseStorage",
            Self::MacHome => "MacHome",
            Self::MacAppSupport => "MacAppSupport",
            Self::MacDocuments => "MacDocuments",
            Self::WinSavedGames => "WinSavedGames",
            Self::WinProgramData => "WinProgramData",
            Self::SteamCloudDocuments => "SteamCloudDocuments",
            Self::WinAppDataLocalLow => "WinAppDataLocalLow",
            Self::MacCaches => "MacCaches",
            Self::LinuxHome => "LinuxHome",
            Self::LinuxXdgDataHome => "LinuxXdgDataHome",
            Self::LinuxXdgConfigHome => "LinuxXdgConfigHome",
            Self::AndroidSteamPackageRoot => "AndroidSteamPackageRoot",
            Self::WindowsHome => "WindowsHome",
        }
    }

    // 从名称字符串解析
    pub fn from_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            // All platforms
            "steamclouddocuments" => Some(Self::SteamCloudDocuments),
            "gameinstall" | "appinstalldirectory" | "app install directory" => {
                Some(Self::GameInstall)
            }
            // Windows
            "winmydocuments" => Some(Self::WinMyDocuments),
            "winappdatalocal" => Some(Self::WinAppDataLocal),
            "winappdataroaming" => Some(Self::WinAppDataRoaming),
            "winappdatalocallow" => Some(Self::WinAppDataLocalLow),
            "winsavedgames" => Some(Self::WinSavedGames),
            "windowshome" => Some(Self::WindowsHome),
            // macOS
            "machome" => Some(Self::MacHome),
            "macappsupport" => Some(Self::MacAppSupport),
            "macdocuments" => Some(Self::MacDocuments),
            // Linux
            "linuxhome" => Some(Self::LinuxHome),
            "linuxxdgdatahome" => Some(Self::LinuxXdgDataHome),
            "linuxxdgconfighome" => Some(Self::LinuxXdgConfigHome),
            _ => None,
        }
    }
}

use anyhow::{Result, anyhow};
use std::path::Path;

// 解析 Root 类型的基础路径
pub fn resolve_root_base_path(
    root_type: RootType,
    steam_path: &Path,
    user_id: &str,
    app_id: u32,
) -> Result<PathBuf> {
    match root_type {
        // Root 0: API 默认路径 (userdata/.../remote/)
        RootType::DefaultApi => Ok(steam_path
            .join("userdata")
            .join(user_id)
            .join(app_id.to_string())
            .join("remote")),

        // Root 1: GameInstall - 需要特殊处理
        RootType::GameInstall => Err(anyhow!("GameInstall 需要特殊处理")),

        // Root 2: WinMyDocuments
        RootType::WinMyDocuments => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("Documents"))
            }
            #[cfg(not(target_os = "windows"))]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Documents"))
            }
        }

        // Root 3: WinAppDataLocal
        RootType::WinAppDataLocal => {
            #[cfg(target_os = "windows")]
            {
                let localappdata = std::env::var("LOCALAPPDATA")?;
                Ok(PathBuf::from(localappdata))
            }
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home))
            }
            #[cfg(target_os = "linux")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join(".local").join("share"))
            }
        }

        // Root 4: WinAppDataRoaming
        RootType::WinAppDataRoaming => {
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

        // Root 5: SteamUserBaseStorage (用途待验证)
        RootType::SteamUserBaseStorage => {
            Err(anyhow!("SteamUserBaseStorage 用途待验证，无法解析路径"))
        }

        // Root 6: MacHome
        RootType::MacHome => {
            let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
            Ok(PathBuf::from(home))
        }

        // Root 7: MacAppSupport
        RootType::MacAppSupport => {
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

        // Root 8: MacDocuments
        RootType::MacDocuments => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("Documents"))
            }
            #[cfg(not(target_os = "windows"))]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Documents"))
            }
        }

        // Root 9: WinSavedGames
        RootType::WinSavedGames => {
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

        // Root 10: WinProgramData
        RootType::WinProgramData => {
            #[cfg(target_os = "windows")]
            {
                let programdata =
                    std::env::var("PROGRAMDATA").unwrap_or_else(|_| r"C:\ProgramData".to_string());
                Ok(PathBuf::from(programdata))
            }
            #[cfg(not(target_os = "windows"))]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join(".local").join("share"))
            }
        }

        // Root 13: MacCaches
        RootType::MacCaches => {
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Library").join("Caches"))
            }
            #[cfg(target_os = "windows")]
            {
                let localappdata = std::env::var("LOCALAPPDATA")?;
                Ok(PathBuf::from(localappdata))
            }
            #[cfg(target_os = "linux")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join(".cache"))
            }
        }

        // Root 17: AndroidSteamPackageRoot
        RootType::AndroidSteamPackageRoot => {
            Err(anyhow!("AndroidSteamPackageRoot 的路径解析尚未实现"))
        }

        // Root 11: SteamCloudDocuments (UFS Auto-Cloud)
        // 完整路径: [base]/[Steam用户名]/[游戏名]/
        RootType::SteamCloudDocuments => get_steam_cloud_docs_dir(steam_path, app_id),

        // Root 12: WinAppDataLocalLow
        RootType::WinAppDataLocalLow => {
            #[cfg(target_os = "windows")]
            {
                let home = std::env::var("USERPROFILE")?;
                Ok(PathBuf::from(home).join("AppData").join("LocalLow"))
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

        // Root 14: LinuxHome
        RootType::LinuxHome => {
            let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
            Ok(PathBuf::from(home))
        }

        // Root 15: LinuxXdgDataHome
        RootType::LinuxXdgDataHome => {
            #[cfg(target_os = "linux")]
            {
                if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
                    Ok(PathBuf::from(xdg))
                } else {
                    let home = std::env::var("HOME")?;
                    Ok(PathBuf::from(home).join(".local").join("share"))
                }
            }
            #[cfg(target_os = "windows")]
            {
                let localappdata = std::env::var("LOCALAPPDATA")?;
                Ok(PathBuf::from(localappdata))
            }
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home)
                    .join("Library")
                    .join("Application Support"))
            }
        }

        // Root 16: LinuxXdgConfigHome
        RootType::LinuxXdgConfigHome => {
            #[cfg(target_os = "linux")]
            {
                if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
                    Ok(PathBuf::from(xdg))
                } else {
                    let home = std::env::var("HOME")?;
                    Ok(PathBuf::from(home).join(".config"))
                }
            }
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
        }

        // Root 18: WindowsHome
        RootType::WindowsHome => {
            let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
            Ok(PathBuf::from(home))
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
        tracing::warn!("未知的 root 值: {}，退回使用 DefaultApi", root);
        RootType::DefaultApi
    });

    // 获取 root 名称
    let root_name = root_type.to_name();

    // 检查是否有 rootoverrides 配置
    if let Some(overrides) = get_root_overrides_cache(app_id)
        && let Some((new_root, add_path, path_transforms)) =
            apply_root_override(root_name, &overrides)
    {
        // 解析新的 root 类型
        if let Some(new_root_type) = RootType::from_name(&new_root) {
            let base_path = resolve_root_base_path(new_root_type, steam_path, user_id, app_id)?;

            // 应用 path_transforms 规则
            let mut final_filename = filename.to_string();
            for transform in &path_transforms {
                if !transform.find.is_empty() {
                    final_filename = final_filename.replace(&transform.find, &transform.replace);
                }
            }

            let final_path = if !path_transforms.is_empty() {
                // 有 pathtransforms 时，应用转换后的路径
                base_path.join(&final_filename)
            } else if !add_path.is_empty() {
                // 无 pathtransforms，有 addpath 时，追加路径
                base_path.join(&add_path).join(filename)
            } else {
                // 都没有时，直接使用 base_path
                base_path.join(filename)
            };
            return Ok(final_path);
        }
    }

    // 无 override，使用默认逻辑
    // GameInstall 需要查找游戏安装目录
    if root_type == RootType::GameInstall {
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
    // 检查缓存
    if let Ok(cache) = GAME_INSTALL_DIR_CACHE.lock()
        && let Some(path) = cache.get(&app_id)
    {
        return Ok(path.clone());
    }

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
                        if line.contains("\"installdir\"")
                            && let Some(dir) = line.split('"').nth(3)
                        {
                            install_dir = Some(dir.to_string());
                        }
                        #[cfg(target_os = "macos")]
                        if line.contains("\"name\"")
                            && let Some(n) = line.split('"').nth(3)
                        {
                            name = Some(n.to_string());
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
                                    // 写入缓存
                                    if let Ok(mut cache) = GAME_INSTALL_DIR_CACHE.lock() {
                                        cache.insert(app_id, app_support_path.clone());
                                    }
                                    return Ok(app_support_path);
                                }
                            }
                        }

                        // 尝试游戏安装目录
                        let install_path = steamapps.join("common").join(&dir);
                        tracing::info!("找到游戏安装目录: {}", install_path.display());
                        // 写入缓存
                        if let Ok(mut cache) = GAME_INSTALL_DIR_CACHE.lock() {
                            cache.insert(app_id, install_path.clone());
                        }
                        return Ok(install_path);
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

// 获取 SteamCloudDocuments 的完整路径
// 路径结构: [base]/[Steam用户名]/[游戏名]/
// 需要从 loginusers.vdf 获取用户名，从 appmanifest 获取游戏名
fn get_steam_cloud_docs_dir(steam_path: &Path, app_id: u32) -> Result<PathBuf> {
    // 检查缓存
    if let Ok(cache) = STEAM_CLOUD_DOCS_CACHE.lock()
        && let Some(path) = cache.get(&app_id)
    {
        return Ok(path.clone());
    }

    // 平台对应的 SteamCloudDocuments 基础目录
    #[cfg(target_os = "macos")]
    let base_dir = {
        let home = std::env::var("HOME")?;
        PathBuf::from(home).join("Documents").join("Steam Cloud")
    };
    #[cfg(target_os = "linux")]
    let base_dir = {
        let home = std::env::var("HOME")?;
        PathBuf::from(home).join(".SteamCloud")
    };
    #[cfg(target_os = "windows")]
    let base_dir = {
        let home = std::env::var("USERPROFILE")?;
        PathBuf::from(home).join("Documents").join("Steam Cloud")
    };

    if !base_dir.exists() {
        return Ok(base_dir);
    }

    // 从 appmanifest 获取游戏名，失败则从缓存获取
    let game_name =
        get_game_name_from_manifest(steam_path, app_id).or_else(|| get_game_name_cached(app_id));

    // 从 loginusers.vdf 获取 Steam 用户名
    let persona_name =
        crate::user_manager::find_user_id_from_loginusers(steam_path).and_then(|(_, name)| name);

    // 优先用精确路径 [base]/[用户名]/[游戏名]/
    if let Some(ref persona) = persona_name
        && let Some(ref gname) = game_name
    {
        let full_path = base_dir.join(persona).join(gname);
        if full_path.exists() {
            tracing::debug!("SteamCloudDocuments 精确匹配: {}", full_path.display());
            if let Ok(mut cache) = STEAM_CLOUD_DOCS_CACHE.lock() {
                cache.insert(app_id, full_path.clone());
            }
            return Ok(full_path);
        }
    }

    // 回退：扫描 base_dir 下的子目录，尝试匹配游戏名
    if let Some(ref gname) = game_name
        && let Ok(entries) = std::fs::read_dir(&base_dir)
    {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let candidate = entry.path().join(gname);
                if candidate.exists() {
                    tracing::debug!("SteamCloudDocuments 目录扫描匹配: {}", candidate.display());
                    if let Ok(mut cache) = STEAM_CLOUD_DOCS_CACHE.lock() {
                        cache.insert(app_id, candidate.clone());
                    }
                    return Ok(candidate);
                }
            }
        }
    }

    // 无法匹配游戏目录，返回错误避免误扫整个基础目录
    Err(anyhow!(
        "无法确定游戏 {} 的 SteamCloudDocuments 目录",
        app_id
    ))
}

// 从 appmanifest 获取游戏显示名称
fn get_game_name_from_manifest(steam_path: &Path, app_id: u32) -> Option<String> {
    let libraries = crate::game_scanner::discover_library_steamapps(steam_path);

    for steamapps in libraries.iter() {
        let manifest_path = steamapps.join(format!("appmanifest_{}.acf", app_id));
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            for line in content.lines() {
                if line.contains("\"name\"")
                    && let Some(name) = line.split('"').nth(3)
                {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

// 收集本地存档路径（使用根基础路径）
// 基于 appinfo.vdf savefiles 配置收集本地存档路径
//
// 逻辑：
// 1. 默认添加 root=0 (SteamRemote) 目录
// 2. 根据 savefiles 配置添加其他 root 类型目录
// 3. 过滤平台不匹配的配置
pub fn collect_local_save_paths_from_ufs(
    savefiles: &[SaveFileConfig],
    steam_path: &Path,
    user_id: &str,
    app_id: u32,
) -> Vec<(String, PathBuf)> {
    use std::collections::HashMap;

    tracing::debug!(
        "开始收集本地存档路径: app_id={}, savefiles={}",
        app_id,
        savefiles.len()
    );

    let mut path_map: HashMap<u32, (String, PathBuf)> = HashMap::new();

    // 默认添加 root=0 (Default API) 目录，即使不存在也显示方便跳转
    let remote_path = steam_path
        .join("userdata")
        .join(user_id)
        .join(app_id.to_string())
        .join("remote");

    let desc = get_root_description(0);
    tracing::debug!("默认路径 root=0: {}", remote_path.display());
    path_map.insert(0, (desc, remote_path));

    // 根据 savefiles 配置添加其他 root 类型目录
    // 预先缓存 game_install_dir，避免重复查找
    let game_install_dir_cache = get_game_install_dir(steam_path, app_id).ok();

    for config in savefiles {
        // 检查平台是否匹配
        if !platform_matches_current(&config.platforms) {
            tracing::debug!(
                "跳过不匹配平台: root={}, platforms={:?}",
                config.root,
                config.platforms
            );
            continue;
        }

        // 获取 root 类型
        let root_type = match &config.root_type {
            Some(rt) => *rt,
            None => {
                tracing::warn!("无法解析 root 类型: {}", config.root);
                continue;
            }
        };

        let root_num = root_type.to_u32();

        // 跳过已经处理过的 root 类型
        if path_map.contains_key(&root_num) {
            continue;
        }

        // 解析基础路径，优先应用 rootoverrides
        let (final_desc, final_path) = {
            let root_name = root_type.to_name();

            if let Some(overrides) = get_root_overrides_cache(app_id)
                && let Some((new_root_name, _add_path, _transforms)) =
                    apply_root_override(root_name, &overrides)
            {
                if let Some(new_root_type) = RootType::from_name(&new_root_name) {
                    let new_root_num = new_root_type.to_u32();
                    let override_path = if new_root_num == 1 {
                        game_install_dir_cache.clone()
                    } else {
                        resolve_root_base_path(new_root_type, steam_path, user_id, app_id).ok()
                    };

                    if let Some(path) = override_path {
                        let desc = format!("{} ({})", new_root_name, root_num);
                        (desc, Some(path))
                    } else {
                        // override 路径解析失败，回退
                        let base = if root_num == 1 {
                            game_install_dir_cache.clone()
                        } else {
                            resolve_root_base_path(root_type, steam_path, user_id, app_id).ok()
                        };
                        (get_root_description(root_num), base)
                    }
                } else {
                    // override 的 root 名称无法解析，回退
                    let base = if root_num == 1 {
                        game_install_dir_cache.clone()
                    } else {
                        resolve_root_base_path(root_type, steam_path, user_id, app_id).ok()
                    };
                    (get_root_description(root_num), base)
                }
            } else {
                // 无 override
                let base = if root_num == 1 {
                    game_install_dir_cache.clone()
                } else {
                    resolve_root_base_path(root_type, steam_path, user_id, app_id).ok()
                };
                (get_root_description(root_num), base)
            }
        };

        if let Some(base_path) = final_path
            && base_path.exists()
        {
            tracing::debug!("✓ {}: {}", final_desc, base_path.display());
            path_map.insert(root_num, (final_desc, base_path));
        }
    }

    let mut sorted_entries: Vec<(u32, (String, PathBuf))> = path_map.into_iter().collect();
    sorted_entries.sort_by_key(|(root_num, _)| *root_num);
    let paths: Vec<(String, PathBuf)> = sorted_entries.into_iter().map(|(_, v)| v).collect();

    if !paths.is_empty() {
        tracing::debug!("检测到 {} 个本地存档根目录", paths.len());
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

// 解析 CDP 格式的 root_description
// 格式: "CDP:<url>|<folder>"
// 返回: (url, folder)
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
// 数字 ID 来源于 remotecache.vdf，名称来源于 Steamworks 文档和 appinfo.vdf
pub fn get_root_type_name(root: u32) -> &'static str {
    match root {
        0 => "Default(API)",
        1 => "GameInstall",
        2 => "WinMyDocuments",
        3 => "WinAppDataLocal",
        4 => "WinAppDataRoaming",
        5 => "SteamUserBaseStorage",
        6 => "MacHome",
        7 => "MacAppSupport",
        8 => "MacDocuments",
        9 => "WinSavedGames",
        10 => "WinProgramData",
        11 => "SteamCloudDocuments",
        12 => "WinAppDataLocalLow",
        13 => "MacCaches",
        14 => "LinuxHome",
        15 => "LinuxXdgDataHome",
        16 => "LinuxXdgConfigHome",
        17 => "AndroidSteamPackageRoot",
        18 => "WindowsHome",
        _ => "Unknown",
    }
}

// 从 appinfo.vdf 的 root 字符串名称转换为 RootType
pub fn root_name_to_type(name: &str) -> Option<RootType> {
    match name.to_lowercase().as_str() {
        // All platforms
        "app install directory" | "gameinstall" | "1" => Some(RootType::GameInstall),
        "steamclouddocuments" | "11" => Some(RootType::SteamCloudDocuments),
        "0" => Some(RootType::DefaultApi),
        // Windows
        "winmydocuments" | "2" => Some(RootType::WinMyDocuments),
        "winappdatalocal" | "3" => Some(RootType::WinAppDataLocal),
        "winappdataroaming" | "4" => Some(RootType::WinAppDataRoaming),
        "winappdatalocallow" | "12" => Some(RootType::WinAppDataLocalLow),
        "winsavedgames" | "9" => Some(RootType::WinSavedGames),
        "windowshome" | "18" => Some(RootType::WindowsHome),
        // macOS
        "machome" | "6" => Some(RootType::MacHome),
        "macappsupport" | "7" => Some(RootType::MacAppSupport),
        "macdocuments" | "8" => Some(RootType::MacDocuments),
        // Linux
        "linuxhome" | "14" => Some(RootType::LinuxHome),
        "linuxxdgdatahome" | "15" => Some(RootType::LinuxXdgDataHome),
        "linuxxdgconfighome" | "16" => Some(RootType::LinuxXdgConfigHome),
        // 其他
        "steamuserbasestorage" | "5" => Some(RootType::SteamUserBaseStorage),
        "winprogramdata" | "10" => Some(RootType::WinProgramData),
        "maccaches" | "13" => Some(RootType::MacCaches),
        "androidsteampackageroot" | "17" => Some(RootType::AndroidSteamPackageRoot),
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

// 应用 Root Override
// 检查给定的 root 名称是否在当前平台上有覆盖配置
// 返回: (新的 root 名称, 附加路径, path_transforms)
pub fn apply_root_override(
    root_name: &str,
    overrides: &[RootOverrideConfig],
) -> Option<(String, String, Vec<PathTransformConfig>)> {
    let current_platform = get_current_platform();

    for override_config in overrides {
        // 检查原始 root 是否匹配
        if override_config.original_root.to_lowercase() != root_name.to_lowercase() {
            continue;
        }

        // 检查 oslist 是否包含当前平台
        let platform_match = override_config.oslist.iter().any(|os| {
            let os_lower = os.to_lowercase();
            match current_platform {
                "windows" => os_lower.contains("windows") || os_lower.contains("win"),
                "macos" => {
                    os_lower.contains("macos")
                        || os_lower.contains("mac")
                        || os_lower.contains("osx")
                }
                "linux" => os_lower.contains("linux"),
                _ => false,
            }
        });

        if platform_match {
            tracing::debug!(
                "应用 Root Override: {} -> {} (platform: {}, addpath: {}, transforms: {})",
                override_config.original_root,
                override_config.new_root,
                current_platform,
                override_config.add_path,
                override_config.path_transforms.len()
            );
            return Some((
                override_config.new_root.clone(),
                override_config.add_path.clone(),
                override_config.path_transforms.clone(),
            ));
        }
    }

    None
}

// 从 ufs savefiles 配置中的路径配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveFileConfig {
    pub root: String,                // root 字符串名称 (如 "WinMyDocuments")
    pub root_type: Option<RootType>, // 解析后的 RootType
    pub path: String,                // 子目录路径
    pub pattern: String,             // 文件匹配模式 (glob)
    pub platforms: Vec<String>,      // 支持的平台
    pub recursive: bool,             // 是否递归 (默认 true)
}

// 路径转换配置 (对应 VDF 中的 pathtransforms)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathTransformConfig {
    pub find: String,    // 要查找的路径片段
    pub replace: String, // 替换为的路径片段
}

// Root Override 配置
// 用于在特定操作系统上将一个根目录重定向到另一个
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RootOverrideConfig {
    pub original_root: String, // 原始根名称 (如 "WinMyDocuments")
    pub oslist: Vec<String>,   // 适用的操作系统列表
    pub new_root: String,      // 新的根名称 (如 "MacDocuments") (VDF: useinstead)
    pub add_path: String,      // 附加路径 (可选) (VDF: addpath)
    pub path_transforms: Vec<PathTransformConfig>, // 路径转换规则 (VDF: pathtransforms)
}

// 扫描到的本地文件信息
#[derive(Debug, Clone)]
pub struct ScannedLocalFile {
    pub relative_path: String, // 相对于 root 的路径 (用于与云端文件名匹配)
    pub root_id: u32,          // 所属 Root ID
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
    let overrides = get_root_overrides_cache(app_id);

    for config in savefiles {
        if !platform_matches_current(&config.platforms) {
            tracing::debug!(
                "跳过不匹配平台的配置: root={}, platforms={:?}",
                config.root,
                config.platforms
            );
            continue;
        }

        let mut root_type = match config.root_type {
            Some(rt) => rt,
            None => {
                tracing::warn!("无法解析 root 类型: {}", config.root);
                continue;
            }
        };

        let mut relative_scan_path = config.path.clone();

        if let Some(ref overrides) = overrides
            && let Some((new_root, add_path, path_transforms)) =
                apply_root_override(&config.root, overrides)
            && let Some(new_rt) = RootType::from_name(&new_root)
        {
            root_type = new_rt;
            tracing::debug!(
                "应用扫描 Override: {} -> {} (path: {})",
                config.root,
                new_root,
                relative_scan_path
            );

            for transform in &path_transforms {
                if !transform.find.is_empty() {
                    relative_scan_path =
                        relative_scan_path.replace(&transform.find, &transform.replace);
                }
            }

            if path_transforms.is_empty() && !add_path.is_empty() {
                if relative_scan_path.is_empty() {
                    relative_scan_path = add_path;
                } else {
                    relative_scan_path = format!("{}/{}", add_path, relative_scan_path);
                }
            }
        }

        let base_path = match resolve_root_base_path(root_type, steam_path, user_id, app_id) {
            Ok(p) => p,
            Err(e) => {
                tracing::debug!("无法解析 root 路径: {} - {}", config.root, e);
                continue;
            }
        };

        let scan_path = if relative_scan_path.is_empty() {
            base_path.clone()
        } else {
            let clean_path = relative_scan_path
                .trim_start_matches('/')
                .trim_start_matches('\\');
            base_path.join(clean_path)
        };

        if !scan_path.exists() {
            tracing::debug!("扫描目录不存在: {}", scan_path.display());
            continue;
        }

        tracing::debug!(
            "扫描本地目录: {} (pattern={}, original_path={})",
            scan_path.display(),
            config.pattern,
            config.path
        );

        let files = scan_directory_with_pattern(&scan_path, &config.pattern, config.recursive);

        for (full_path, relative_to_scan) in files {
            // cloud_relative_path 使用原始 config.path 拼接，用于匹配云端文件名
            let cloud_relative_path = if config.path.is_empty() {
                relative_to_scan
            } else {
                let clean_path = config.path.trim_start_matches('/').trim_start_matches('\\');
                format!("{}/{}", clean_path, relative_to_scan)
            };

            if let Ok(metadata) = std::fs::metadata(&full_path) {
                results.push(ScannedLocalFile {
                    relative_path: cloud_relative_path,
                    root_id: config.root_type.unwrap().to_u32(),
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
            } else if path.is_file()
                && let Some(filename) = path.file_name().and_then(|n| n.to_str())
                && pattern_matches(filename, pattern)
                && let Ok(rel) = path.strip_prefix(base)
            {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                results.push((path.clone(), rel_str));
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
