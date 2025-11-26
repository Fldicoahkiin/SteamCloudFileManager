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
    // 7: 视频文件夹 (Win/Linux: Videos, Mac: Movies)
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

    // 获取描述文本
    pub fn description(&self) -> &'static str {
        match self {
            Self::SteamRemote => "Steam云文件夹 (Remote)",
            Self::GameInstallDir => "游戏安装目录",
            Self::Documents => {
                #[cfg(target_os = "windows")]
                return "我的文档 (Documents)";
                #[cfg(target_os = "macos")]
                return "文稿 (Documents)";
                #[cfg(target_os = "linux")]
                return "文档 (Documents)";
            }
            Self::AppDataRoaming => {
                #[cfg(target_os = "windows")]
                return "AppData Roaming";
                #[cfg(target_os = "macos")]
                return "Application Support";
                #[cfg(target_os = "linux")]
                return ".config";
            }
            Self::AppDataLocal => {
                #[cfg(target_os = "windows")]
                return "AppData Local";
                #[cfg(target_os = "macos")]
                return "Caches";
                #[cfg(target_os = "linux")]
                return ".local/share";
            }
            Self::Pictures => "图片文件夹",
            Self::Music => "音乐文件夹",
            Self::Videos => {
                #[cfg(target_os = "macos")]
                return "影片 (Movies)";
                #[cfg(not(target_os = "macos"))]
                return "视频 (Videos)";
            }
            Self::Desktop => "桌面文件夹",
            Self::SavedGames => "Windows Saved Games",
            Self::Downloads => "下载文件夹",
            Self::PublicShared => "公共共享目录",
            Self::AppDataLocalLow => "Windows LocalLow",
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
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join("Movies"))
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
fn get_game_install_dir(steam_path: &Path, app_id: u32) -> Result<PathBuf> {
    for steamapps in crate::game_scanner::discover_library_steamapps(steam_path) {
        let manifest_path = steamapps.join(format!("appmanifest_{}.acf", app_id));
        if manifest_path.exists() {
            let content = std::fs::read_to_string(&manifest_path)?;
            for line in content.lines() {
                if line.contains("\"installdir\"") {
                    if let Some(dir) = line.split('"').nth(3) {
                        return Ok(steamapps.join("common").join(dir));
                    }
                }
            }
        }
    }

    // 如果找不到，返回默认路径
    Ok(steam_path
        .join("steamapps")
        .join("common")
        .join(format!("Game_{}", app_id)))
}
