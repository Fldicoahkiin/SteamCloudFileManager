use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// 链接方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkDirection {
    // 本地目录(源) → remote(链接)
    // 在 remote 目录创建指向本地目录的软链接，Steam 同步本地文件到云端
    RemoteToLocal,
    // remote(源) → 本地目录(链接)
    // 在本地目录创建指向 remote 的软链接，应用程序访问云端文件
    LocalToRemote,
}

impl LinkDirection {
    pub fn description(&self) -> &'static str {
        match self {
            LinkDirection::RemoteToLocal => "本地目录(源) → remote(链接)",
            LinkDirection::LocalToRemote => "remote(源) → 本地目录(链接)",
        }
    }
}

// 软链接状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkStatus {
    // 有效：链接存在且指向正确目标
    Valid,
    // 断开：链接存在但目标不存在
    Broken,
    // 不存在：链接未创建
    NotExists,
    // 冲突：链接位置已存在普通文件/目录
    Conflict,
}

impl LinkStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            LinkStatus::Valid => "✅",
            LinkStatus::Broken => "⚠️",
            LinkStatus::NotExists => "❌",
            LinkStatus::Conflict => "🔴",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            LinkStatus::Valid => "链接有效",
            LinkStatus::Broken => "链接断开",
            LinkStatus::NotExists => "未创建",
            LinkStatus::Conflict => "路径冲突",
        }
    }
}

// 软链接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkConfig {
    // 唯一标识
    pub id: String,
    // 游戏 App ID
    pub app_id: u32,
    // 链接方向
    pub direction: LinkDirection,
    // 用户自定义的本地路径
    pub local_path: PathBuf,
    // remote 目录下的子文件夹名
    pub remote_subfolder: String,
    // 创建时的平台
    pub platform: String,
    // 创建时间 (Unix timestamp)
    pub created_at: i64,
    // 备注
    #[serde(default)]
    pub note: String,
}

impl SymlinkConfig {
    // 创建新配置
    pub fn new(
        app_id: u32,
        direction: LinkDirection,
        local_path: PathBuf,
        remote_subfolder: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            app_id,
            direction,
            local_path,
            remote_subfolder,
            platform: get_current_platform().to_string(),
            created_at: chrono::Utc::now().timestamp(),
            note: String::new(),
        }
    }

    // 获取链接路径（软链接本身的位置）
    pub fn get_link_path(&self, remote_dir: &Path) -> PathBuf {
        match self.direction {
            LinkDirection::RemoteToLocal => remote_dir.join(&self.remote_subfolder),
            LinkDirection::LocalToRemote => self.local_path.clone(),
        }
    }

    // 获取目标路径（软链接指向的位置）
    pub fn get_target_path(&self, remote_dir: &Path) -> PathBuf {
        match self.direction {
            LinkDirection::RemoteToLocal => self.local_path.clone(),
            LinkDirection::LocalToRemote => remote_dir.join(&self.remote_subfolder),
        }
    }
}

// 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkConfigFile {
    pub version: u32,
    pub configs: Vec<SymlinkConfig>,
}

impl Default for SymlinkConfigFile {
    fn default() -> Self {
        Self {
            version: 1,
            configs: Vec::new(),
        }
    }
}

// 软链接管理器
pub struct SymlinkManager {
    steam_path: PathBuf,
    user_id: String,
    config_path: PathBuf,
}

impl SymlinkManager {
    // 创建管理器
    pub fn new(steam_path: PathBuf, user_id: String) -> Result<Self> {
        let config_path = get_config_path()?;
        Ok(Self {
            steam_path,
            user_id,
            config_path,
        })
    }

    // 获取游戏的 remote 目录
    pub fn get_remote_dir(&self, app_id: u32) -> PathBuf {
        self.steam_path
            .join("userdata")
            .join(&self.user_id)
            .join(app_id.to_string())
            .join("remote")
    }

    // 创建软链接
    pub fn create_symlink(&self, config: &SymlinkConfig) -> Result<()> {
        let remote_dir = self.get_remote_dir(config.app_id);
        let link_path = config.get_link_path(&remote_dir);
        let target_path = config.get_target_path(&remote_dir);

        // 验证目标路径存在
        if !target_path.exists() {
            // 对于 RemoteToLocal，创建目标目录
            if config.direction == LinkDirection::RemoteToLocal {
                return Err(anyhow!("本地目录不存在: {:?}", target_path));
            }
            // 对于 LocalToRemote，创建 remote 子目录
            fs::create_dir_all(&target_path)?;
        }

        // 检查链接位置是否已存在
        if link_path.exists() || link_path.symlink_metadata().is_ok() {
            return Err(anyhow!("链接位置已存在: {:?}", link_path));
        }

        // 确保链接的父目录存在
        if let Some(parent) = link_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 创建软链接
        create_symlink_platform(&target_path, &link_path)?;

        tracing::info!("创建软链接: {:?} → {:?}", link_path, target_path);

        Ok(())
    }

    // 删除软链接
    pub fn remove_symlink(&self, config: &SymlinkConfig) -> Result<()> {
        let remote_dir = self.get_remote_dir(config.app_id);
        let link_path = config.get_link_path(&remote_dir);

        // 检查是否是软链接
        if let Ok(metadata) = link_path.symlink_metadata() {
            if metadata.file_type().is_symlink() {
                remove_symlink_platform(&link_path)?;
                tracing::info!("删除软链接: {:?}", link_path);
                return Ok(());
            }
        }

        Err(anyhow!("路径不是软链接: {:?}", link_path))
    }

    // 验证软链接状态
    pub fn verify_symlink(&self, config: &SymlinkConfig) -> LinkStatus {
        let remote_dir = self.get_remote_dir(config.app_id);
        let link_path = config.get_link_path(&remote_dir);
        let target_path = config.get_target_path(&remote_dir);

        // 检查链接路径
        match link_path.symlink_metadata() {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    // 是软链接，检查目标
                    if let Ok(resolved) = fs::read_link(&link_path) {
                        // 比较解析后的路径是否指向正确目标
                        let resolved_abs = if resolved.is_absolute() {
                            resolved
                        } else {
                            link_path.parent().unwrap_or(Path::new("")).join(&resolved)
                        };

                        if resolved_abs == target_path
                            || fs::canonicalize(&resolved_abs).ok()
                                == fs::canonicalize(&target_path).ok()
                        {
                            if target_path.exists() {
                                LinkStatus::Valid
                            } else {
                                LinkStatus::Broken
                            }
                        } else {
                            LinkStatus::Broken
                        }
                    } else {
                        LinkStatus::Broken
                    }
                } else {
                    // 不是软链接，是普通文件/目录
                    LinkStatus::Conflict
                }
            }
            Err(_) => LinkStatus::NotExists,
        }
    }

    // 生成手动执行的命令
    pub fn generate_commands(&self, config: &SymlinkConfig) -> Vec<String> {
        let remote_dir = self.get_remote_dir(config.app_id);
        let link_path = config.get_link_path(&remote_dir);
        let target_path = config.get_target_path(&remote_dir);

        generate_symlink_commands(&target_path, &link_path)
    }

    // 加载配置
    pub fn load_configs(&self) -> Result<SymlinkConfigFile> {
        if !self.config_path.exists() {
            return Ok(SymlinkConfigFile::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let config: SymlinkConfigFile = serde_json::from_str(&content)?;
        Ok(config)
    }

    // 保存配置
    pub fn save_configs(&self, configs: &SymlinkConfigFile) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(configs)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    // 添加配置
    pub fn add_config(&self, config: SymlinkConfig) -> Result<()> {
        let mut file = self.load_configs()?;
        file.configs.push(config);
        self.save_configs(&file)
    }

    // 删除配置
    pub fn remove_config(&self, id: &str) -> Result<()> {
        let mut file = self.load_configs()?;
        file.configs.retain(|c| c.id != id);
        self.save_configs(&file)
    }

    // 获取指定游戏的配置
    pub fn get_configs_for_app(&self, app_id: u32) -> Result<Vec<SymlinkConfig>> {
        let file = self.load_configs()?;
        Ok(file
            .configs
            .into_iter()
            .filter(|c| c.app_id == app_id)
            .collect())
    }
}

// 获取配置文件路径
fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .or_else(dirs::data_local_dir)
        .ok_or_else(|| anyhow!("无法获取配置目录"))?;

    Ok(config_dir.join("scfm").join("symlink_configs.json"))
}

// 获取当前平台名称
fn get_current_platform() -> &'static str {
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
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "unknown"
    }
}

// 跨平台创建软链接
#[cfg(unix)]
fn create_symlink_platform(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link)?;
    Ok(())
}

#[cfg(windows)]
fn create_symlink_platform(target: &Path, link: &Path) -> Result<()> {
    // Windows 上需要管理员权限或开发者模式
    if target.is_dir() {
        std::os::windows::fs::symlink_dir(target, link)?;
    } else {
        std::os::windows::fs::symlink_file(target, link)?;
    }
    Ok(())
}

// 跨平台删除软链接
#[cfg(unix)]
fn remove_symlink_platform(link: &Path) -> Result<()> {
    fs::remove_file(link)?;
    Ok(())
}

#[cfg(windows)]
fn remove_symlink_platform(link: &Path) -> Result<()> {
    // Windows 上需要判断链接是文件还是目录类型
    let metadata = link.symlink_metadata()?;
    if metadata.is_dir() {
        fs::remove_dir(link)?;
    } else {
        fs::remove_file(link)?;
    }
    Ok(())
}

// 生成平台特定的软链接命令
fn generate_symlink_commands(target: &Path, link: &Path) -> Vec<String> {
    let target_str = target.to_string_lossy();
    let link_str = link.to_string_lossy();

    #[cfg(target_os = "macos")]
    {
        vec![
            format!("# macOS 创建软链接"),
            format!("ln -s \"{}\" \"{}\"", target_str, link_str),
            format!(""),
            format!("# 删除软链接"),
            format!("rm \"{}\"", link_str),
        ]
    }

    #[cfg(target_os = "linux")]
    {
        vec![
            format!("# Linux 创建软链接"),
            format!("ln -s \"{}\" \"{}\"", target_str, link_str),
            format!(""),
            format!("# 删除软链接"),
            format!("rm \"{}\"", link_str),
        ]
    }

    #[cfg(target_os = "windows")]
    {
        vec![
            format!("# Windows PowerShell (需要管理员权限)"),
            format!(
                "New-Item -ItemType SymbolicLink -Path \"{}\" -Target \"{}\"",
                link_str, target_str
            ),
            format!(""),
            format!("# Windows CMD (需要管理员权限)"),
            format!("mklink /D \"{}\" \"{}\"", link_str, target_str),
            format!(""),
            format!("# Windows Junction (无需管理员权限，仅目录)"),
            format!("mklink /J \"{}\" \"{}\"", link_str, target_str),
            format!(""),
            format!("# 删除软链接"),
            format!("rmdir \"{}\"", link_str),
        ]
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        vec![format!("# 不支持的平台")]
    }
}
