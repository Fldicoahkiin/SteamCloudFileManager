use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

// 全局配置实例
static CONFIG: OnceLock<Mutex<AppConfig>> = OnceLock::new();

// 应用配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    // 路径设置
    #[serde(default)]
    pub paths: PathsConfig,

    // 外观设置
    #[serde(default)]
    pub appearance: AppearanceConfig,

    // 日志设置
    #[serde(default)]
    pub logging: LoggingConfig,

    // 软链接配置
    #[serde(default)]
    pub symlinks: Vec<SymlinkConfigEntry>,
}

// 软链接配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkConfigEntry {
    pub id: String,
    pub app_id: u32,
    pub direction: String, // "remote_to_local" or "local_to_remote"
    pub local_path: PathBuf,
    pub remote_subfolder: String,
    #[serde(default)]
    pub platform: String, // "windows", "macos", "linux"
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub note: String,
}

// 单个 savefile 条目
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveFileEntry {
    pub root: String,    // Root 类型名称
    pub path: String,    // 相对路径
    pub pattern: String, // 文件匹配模式
    #[serde(default)]
    pub platforms: Vec<String>, // 支持的平台
    #[serde(default = "default_recursive")]
    pub recursive: bool, // 是否递归
}

fn default_recursive() -> bool {
    true
}

// 路径转换规则 (对应 VDF 中的 pathtransforms)
// Steamworks UI 中勾选 "Replace Path" 时会生成此结构
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PathTransform {
    pub find: String,    // 要查找的路径片段
    pub replace: String, // 替换为的路径片段
}

// 单个 rootoverride 条目
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RootOverrideEntry {
    pub original_root: String, // 原始根名称 (VDF: root)
    pub os: String,            // 目标操作系统 (VDF: os)
    pub new_root: String,      // 新的根名称 (VDF: useinstead)
    #[serde(default)]
    pub add_path: String, // 附加路径 (VDF: addpath), 当无 pathtransforms 时使用
    #[serde(default)]
    pub path_transforms: Vec<PathTransform>, // 路径转换规则 (VDF: pathtransforms)
                               // 注意：Steamworks 中 "Replace Path" 勾选框的实际行为是：
                               // - 勾选: 生成 pathtransforms 结构，不生成 addpath
                               // - 不勾选: 生成 addpath 字段，不生成 pathtransforms
}

// 游戏 UFS 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UfsGameConfig {
    pub app_id: u32,
    pub savefiles: Vec<SaveFileEntry>,
    #[serde(default)]
    pub root_overrides: Vec<RootOverrideEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathsConfig {
    // 自定义 Steam 路径（None = 自动检测）
    pub steam_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    // 主题模式: "system", "light", "dark"
    #[serde(default = "default_theme")]
    pub theme_mode: String,

    // 语言设置: "auto", "zh", "en"
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_language() -> String {
    "auto".to_string()
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme_mode: default_theme(),
            language: default_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    // 是否启用日志文件存储
    #[serde(default = "default_log_enabled")]
    pub enabled: bool,
}

fn default_log_enabled() -> bool {
    true
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: default_log_enabled(),
        }
    }
}

impl AppConfig {
    // 生成默认配置文件内容
    pub fn default_toml_with_comments() -> String {
        include_str!("../assets/config_template.toml").to_string()
    }
}

// 获取配置目录
// - macOS: ~/Library/Application Support/SteamCloudFileManager
// - Windows: 应用所在目录
// - Linux: ~/.config/SteamCloudFileManager
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = if cfg!(target_os = "macos") {
        let home = std::env::var("HOME")?;
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("SteamCloudFileManager")
    } else if cfg!(target_os = "windows") {
        // Windows: 使用应用所在目录（便携模式）
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                exe_dir.to_path_buf()
            } else {
                // 降级到 LOCALAPPDATA
                let appdata = std::env::var("LOCALAPPDATA")?;
                PathBuf::from(appdata).join("SteamCloudFileManager")
            }
        } else {
            // 降级到 LOCALAPPDATA
            let appdata = std::env::var("LOCALAPPDATA")?;
            PathBuf::from(appdata).join("SteamCloudFileManager")
        }
    } else {
        let home = std::env::var("HOME")?;
        PathBuf::from(home)
            .join(".config")
            .join("SteamCloudFileManager")
    };

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

// 获取配置文件路径
pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("config.toml"))
}

// 加载配置
pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;

    // 如果配置文件存在，加载它
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: AppConfig = toml::from_str(&content)?;
        tracing::debug!("加载配置文件: {:?}", config_path);
        return Ok(config);
    }

    // 配置文件不存在，创建默认配置
    let config = AppConfig::default();
    save_config(&config)?;

    Ok(config)
}

// 保存配置
pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path()?;

    // 如果是默认配置，写入带注释的版本
    let content = if config.paths.steam_path.is_none()
        && config.appearance.theme_mode == "system"
        && config.appearance.language == "auto"
        && config.logging.enabled
    {
        AppConfig::default_toml_with_comments()
    } else {
        toml::to_string_pretty(config)?
    };

    std::fs::write(&config_path, content)?;
    tracing::debug!("保存配置文件: {:?}", config_path);

    Ok(())
}

// 初始化全局配置
pub fn init_config() -> Result<()> {
    let config = load_config()?;
    let _ = CONFIG.set(Mutex::new(config));
    Ok(())
}

// 获取当前配置的克隆
pub fn get_config() -> AppConfig {
    CONFIG
        .get()
        .and_then(|m| m.lock().ok())
        .map(|c| c.clone())
        .unwrap_or_default()
}

// 更新配置
pub fn update_config<F>(f: F) -> Result<()>
where
    F: FnOnce(&mut AppConfig),
{
    if let Some(mutex) = CONFIG.get()
        && let Ok(mut config) = mutex.lock()
    {
        f(&mut config);
        save_config(&config)?;
    }
    Ok(())
}

// 重置为默认配置
pub fn reset_to_default() -> Result<()> {
    let default_config = AppConfig::default();
    if let Some(mutex) = CONFIG.get()
        && let Ok(mut config) = mutex.lock()
    {
        *config = default_config.clone();
    }
    save_config(&default_config)?;
    tracing::info!("配置已重置为默认值");
    Ok(())
}

// 获取自定义 Steam 路径
pub fn get_custom_steam_path() -> Option<PathBuf> {
    get_config().paths.steam_path
}

// 设置自定义 Steam 路径
pub fn set_custom_steam_path(path: Option<PathBuf>) -> Result<()> {
    update_config(|config| {
        config.paths.steam_path = path;
    })
}

// 验证 Steam 路径是否有效
pub fn validate_steam_path(path: &std::path::Path) -> SteamPathValidation {
    if !path.exists() {
        return SteamPathValidation::NotExists;
    }

    let userdata = path.join("userdata");
    if !userdata.exists() {
        return SteamPathValidation::InvalidStructure;
    }

    // 统计用户数量
    let user_count = std::fs::read_dir(&userdata)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                        && e.file_name()
                            .to_str()
                            .map(|s| s.chars().all(|c| c.is_ascii_digit()))
                            .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0);

    if user_count == 0 {
        return SteamPathValidation::NoUsers;
    }

    SteamPathValidation::Valid { user_count }
}

// Steam 路径验证结果
#[derive(Debug, Clone, PartialEq)]
pub enum SteamPathValidation {
    // 路径有效
    Valid { user_count: usize },
    // 路径不存在
    NotExists,
    // 路径存在但结构无效（缺少 userdata）
    InvalidStructure,
    // 没有找到用户
    NoUsers,
}

impl SteamPathValidation {
    pub fn is_valid(&self) -> bool {
        matches!(self, SteamPathValidation::Valid { .. })
    }
}

// 获取软链接配置
pub fn get_symlink_configs() -> Vec<SymlinkConfigEntry> {
    get_config().symlinks
}

// 获取指定游戏的软链接配置
pub fn get_symlink_configs_for_app(app_id: u32) -> Vec<SymlinkConfigEntry> {
    get_symlink_configs()
        .into_iter()
        .filter(|c| c.app_id == app_id)
        .collect()
}

// 添加软链接配置
pub fn add_symlink_config(entry: SymlinkConfigEntry) -> Result<()> {
    let config = CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("配置未初始化"))?;
    let mut config = config.lock().map_err(|_| anyhow::anyhow!("配置锁定失败"))?;
    config.symlinks.push(entry);

    // 保存到文件
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(&*config)?;
    std::fs::write(&config_path, content)?;
    Ok(())
}

// 删除软链接配置
pub fn remove_symlink_config(id: &str) -> Result<()> {
    let config = CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("配置未初始化"))?;
    let mut config = config.lock().map_err(|_| anyhow::anyhow!("配置锁定失败"))?;
    config.symlinks.retain(|c| c.id != id);

    // 保存到文件
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(&*config)?;
    std::fs::write(&config_path, content)?;
    Ok(())
}

// ============== UFS 游戏配置管理函数 ==============

// 获取 UFS 配置目录
pub fn get_ufs_config_dir() -> Result<PathBuf> {
    let dir = get_config_dir()?.join("ufs_configs");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

// 获取指定游戏的 UFS 配置
pub fn get_ufs_game_config(app_id: u32) -> Option<UfsGameConfig> {
    let dir = get_ufs_config_dir().ok()?;
    let path = dir.join(format!("{}.json", app_id));
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

// 保存或更新 UFS 游戏配置
pub fn save_ufs_game_config(entry: &UfsGameConfig) -> Result<()> {
    let dir = get_ufs_config_dir()?;
    let path = dir.join(format!("{}.json", entry.app_id));
    let content = serde_json::to_string_pretty(entry)?;
    std::fs::write(&path, content)?;
    tracing::info!("已保存 UFS 配置: {}", path.display());
    Ok(())
}

// 删除 UFS 游戏配置
pub fn remove_ufs_game_config(app_id: u32) -> Result<()> {
    let dir = get_ufs_config_dir()?;
    let path = dir.join(format!("{}.json", app_id));
    if path.exists() {
        std::fs::remove_file(&path)?;
        tracing::info!("已删除 UFS 配置: {}", path.display());
    }
    Ok(())
}
