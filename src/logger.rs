// 日志管理模块
use anyhow::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

// 全局日志启用状态
static LOG_ENABLED: AtomicBool = AtomicBool::new(true);
// 当前配置状态
static CURRENT_CONFIG: AtomicBool = AtomicBool::new(true);

// 检查日志是否启用
pub fn is_log_enabled() -> bool {
    CURRENT_CONFIG.load(Ordering::Relaxed)
}

// 检查日志配置是否已更改（需要重启）
pub fn is_log_config_changed() -> bool {
    LOG_ENABLED.load(Ordering::Relaxed) != CURRENT_CONFIG.load(Ordering::Relaxed)
}

// 设置日志启用状态
pub fn set_log_enabled(enabled: bool) {
    CURRENT_CONFIG.store(enabled, Ordering::Relaxed);

    // 保存到配置文件
    if let Err(e) = save_log_config(enabled) {
        tracing::warn!("保存日志配置失败: {}", e);
    }
}

// 获取配置目录
fn get_config_dir() -> Result<PathBuf> {
    let config_dir = if cfg!(target_os = "macos") {
        let home = std::env::var("HOME")?;
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("SteamCloudFileManager")
    } else if cfg!(target_os = "windows") {
        let appdata = std::env::var("LOCALAPPDATA")?;
        PathBuf::from(appdata).join("SteamCloudFileManager")
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

// 保存日志配置
fn save_log_config(enabled: bool) -> Result<()> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join("log_config.txt");
    std::fs::write(config_file, if enabled { "enabled" } else { "disabled" })?;
    Ok(())
}

// 加载日志配置
fn load_log_config() -> bool {
    if let Ok(config_dir) = get_config_dir() {
        let config_file = config_dir.join("log_config.txt");
        if let Ok(content) = std::fs::read_to_string(config_file) {
            return content.trim() == "enabled";
        }
    }
    true // 默认启用
}

// 获取日志目录路径（系统标准目录）
pub fn get_log_dir() -> Result<PathBuf> {
    let log_dir = if cfg!(target_os = "macos") {
        let home = std::env::var("HOME")?;
        PathBuf::from(home)
            .join("Library")
            .join("Logs")
            .join("SteamCloudFileManager")
    } else if cfg!(target_os = "windows") {
        let appdata = std::env::var("LOCALAPPDATA")?;
        PathBuf::from(appdata)
            .join("SteamCloudFileManager")
            .join("logs")
    } else {
        // Linux
        let home = std::env::var("HOME")?;
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("SteamCloudFileManager")
            .join("logs")
    };

    // 确保目录存在
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)?;
    }

    Ok(log_dir)
}

// 获取当前日志文件路径
pub fn get_current_log_file() -> Result<PathBuf> {
    let log_dir = get_log_dir()?;
    let log_file = log_dir.join("app.log");
    Ok(log_file)
}

// 打开日志目录
pub fn open_log_directory() -> Result<()> {
    let log_dir = get_log_dir()?;

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&log_dir).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&log_dir)
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        // 尝试多个文件管理器
        let managers = ["xdg-open", "nautilus", "dolphin", "thunar"];
        let mut success = false;

        for manager in &managers {
            if let Ok(_) = std::process::Command::new(manager).arg(&log_dir).spawn() {
                success = true;
                break;
            }
        }

        if !success {
            return Err(anyhow::anyhow!("无法找到文件管理器"));
        }
    }

    Ok(())
}

// 初始化日志系统（输出到文件和控制台）
pub fn init_logger() -> Result<()> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // 加载日志配置
    let log_enabled = load_log_config();
    LOG_ENABLED.store(log_enabled, Ordering::Relaxed);
    CURRENT_CONFIG.store(log_enabled, Ordering::Relaxed);

    // 环境过滤器（总是使用，确保控制台输出）
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        "info,SteamCloudFileManager=debug,ureq=warn,rustls=warn,tungstenite=warn".into()
    });

    // 控制台输出层（总是启用）
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_line_number(true)
        .with_thread_ids(false);

    // 如果启用日志文件，添加文件输出层
    if log_enabled {
        let log_file = get_current_log_file()?;

        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::sync::Arc::new(file))
            .with_ansi(false)
            .with_target(true)
            .with_line_number(true)
            .with_thread_ids(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .with(file_layer)
            .init();

        tracing::info!("日志文件: {}", log_file.display());
    } else {
        // 只有控制台输出
        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .init();

        tracing::info!("日志文件存储已禁用");
    }

    Ok(())
}
