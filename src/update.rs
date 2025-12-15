use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

const GITHUB_API_RELEASES: &str =
    "https://api.github.com/repos/Fldicoahkiin/SteamCloudFileManager/releases/latest";
const GITHUB_REPO_URL: &str = "https://github.com/Fldicoahkiin/SteamCloudFileManager";

// GitHub Release 信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub html_url: String,
    pub assets: Vec<ReleaseAsset>,
}

// Release 资源文件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

// 更新状态
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    Available(ReleaseInfo),
    NoUpdate,
    Downloading(f32), // 下载进度 0.0-1.0
    #[allow(dead_code)]
    Installing,
    #[allow(dead_code)]
    Success,
    Error(String),
}

// 更新管理器
pub struct UpdateManager {
    status: UpdateStatus,
    current_version: String,
}

impl UpdateManager {
    pub fn new() -> Self {
        Self {
            status: UpdateStatus::Idle,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn status(&self) -> &UpdateStatus {
        &self.status
    }

    // 检查更新
    pub fn check_update(&mut self) -> Result<()> {
        self.status = UpdateStatus::Checking;
        tracing::info!("开始检查更新...");

        match self.fetch_latest_release() {
            Ok(release) => {
                let latest_version = release.tag_name.trim_start_matches('v');
                tracing::info!(
                    "最新版本: {}, 当前版本: {}",
                    latest_version,
                    self.current_version
                );

                if Self::is_newer_version(latest_version, &self.current_version) {
                    tracing::info!("发现新版本: {}", latest_version);
                    self.status = UpdateStatus::Available(release);
                } else {
                    tracing::info!("当前已是最新版本");
                    self.status = UpdateStatus::NoUpdate;
                }
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("检查更新失败: {}", e);
                tracing::error!("{}", err_msg);
                self.status = UpdateStatus::Error(err_msg.clone());
                Err(anyhow!(err_msg))
            }
        }
    }

    // 获取最新 Release 信息
    fn fetch_latest_release(&self) -> Result<ReleaseInfo> {
        let response = ureq::get(GITHUB_API_RELEASES)
            .call()
            .map_err(|e| anyhow!("请求失败: {}", e))?;

        let release: ReleaseInfo = serde_json::from_reader(response.into_body().into_reader())
            .map_err(|e| anyhow!("解析响应失败: {}", e))?;

        Ok(release)
    }

    // 比较版本号
    fn is_newer_version(latest: &str, current: &str) -> bool {
        let parse_version =
            |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect() };

        let latest_parts = parse_version(latest);
        let current_parts = parse_version(current);

        for i in 0..latest_parts.len().max(current_parts.len()) {
            let latest_part = latest_parts.get(i).unwrap_or(&0);
            let current_part = current_parts.get(i).unwrap_or(&0);

            if latest_part > current_part {
                return true;
            } else if latest_part < current_part {
                return false;
            }
        }

        false
    }

    // 启动异步下载
    pub fn start_download(&mut self, release: &ReleaseInfo) -> Receiver<Result<PathBuf, String>> {
        tracing::info!("开始异步下载更新: {}", release.tag_name);

        // 打印下载目录
        if let Ok(update_dir) = Self::get_update_dir() {
            tracing::info!("下载目录: {}", update_dir.display());
        }

        let (tx, rx) = channel();
        let release_clone = release.clone();

        std::thread::spawn(move || {
            let result = Self::download_in_background(&release_clone);
            let _ = tx.send(result);
        });

        self.status = UpdateStatus::Downloading(0.0);
        rx
    }

    // 后台下载
    fn download_in_background(release: &ReleaseInfo) -> Result<PathBuf, String> {
        // 根据平台选择对应的资源文件
        let asset = Self::select_asset_for_platform(&release.assets).map_err(|e| e.to_string())?;
        tracing::info!("选择资源文件: {}", asset.name);

        // 下载文件
        let update_dir = Self::get_update_dir().map_err(|e| e.to_string())?;
        let download_path = update_dir.join(&asset.name);

        tracing::info!("下载文件: {}", asset.name);
        tracing::info!("下载地址: {}", asset.browser_download_url);
        tracing::info!("保存路径: {}", download_path.display());
        tracing::info!("文件大小: {} MB", asset.size as f64 / 1024.0 / 1024.0);

        tracing::info!("开始下载...");

        // 如果文件已存在，先删除
        if download_path.exists() {
            tracing::info!("删除已存在的旧文件");
            fs::remove_file(&download_path).map_err(|e| e.to_string())?;
        }

        let response = ureq::get(&asset.browser_download_url).call().map_err(|e| {
            tracing::error!("HTTP 请求失败: {}", e);
            format!("下载失败: {}", e)
        })?;

        let mut file = fs::File::create(&download_path).map_err(|e| e.to_string())?;
        let mut reader = response.into_body().into_reader();

        std::io::copy(&mut reader, &mut file).map_err(|e| {
            tracing::error!("写入文件失败: {}", e);
            format!("写入文件失败: {}", e)
        })?;

        tracing::info!("下载完成: {}", download_path.display());
        Ok(download_path)
    }

    // 安装已下载的更新
    #[allow(dead_code)]
    pub fn install_downloaded_update(&mut self, download_path: &PathBuf) -> Result<()> {
        self.status = UpdateStatus::Installing;
        self.install_update(download_path)?;
        self.status = UpdateStatus::Success;
        tracing::info!("更新安装成功");
        Ok(())
    }

    // 根据平台选择资源文件
    fn select_asset_for_platform(assets: &[ReleaseAsset]) -> Result<ReleaseAsset> {
        let platform = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        let pattern = match (platform, arch) {
            ("macos", "x86_64") => "macos-x86_64",
            ("macos", "aarch64") => "macos-aarch64",
            ("windows", "x86_64") => "windows-x86_64",
            ("linux", "x86_64") => "linux-x86_64",
            _ => return Err(anyhow!("不支持的平台: {} ({})", platform, arch)),
        };

        assets
            .iter()
            .find(|a| a.name.contains(pattern))
            .cloned()
            .ok_or_else(|| anyhow!("未找到适合当前平台的安装包"))
    }

    // 获取更新下载目录
    pub fn get_update_dir() -> Result<PathBuf> {
        let update_dir = if cfg!(target_os = "macos") {
            let home = std::env::var("HOME")?;
            PathBuf::from(home)
                .join("Library")
                .join("Caches")
                .join("SteamCloudFileManager")
                .join("updates")
        } else if cfg!(target_os = "windows") {
            let appdata = std::env::var("LOCALAPPDATA")?;
            PathBuf::from(appdata)
                .join("SteamCloudFileManager")
                .join("updates")
        } else {
            let home = std::env::var("HOME")?;
            PathBuf::from(home)
                .join(".cache")
                .join("SteamCloudFileManager")
                .join("updates")
        };

        if !update_dir.exists() {
            fs::create_dir_all(&update_dir)?;
        }

        Ok(update_dir)
    }

    // 安装更新
    #[allow(dead_code)]
    fn install_update(&self, download_path: &PathBuf) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            // macOS: DMG 需要手动安装，打开 Finder 显示文件
            tracing::info!("macOS 更新已下载到: {}", download_path.display());
            tracing::info!("请手动打开 DMG 文件并拖拽应用到 Applications 文件夹");

            // 打开 Finder 显示下载的 DMG
            if let Err(e) = std::process::Command::new("open")
                .arg("-R")
                .arg(download_path)
                .spawn()
            {
                tracing::error!("无法打开 Finder: {}", e);
            }

            Err(anyhow!("macOS 需要手动安装 DMG 文件"))
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: 使用系统命令解压 ZIP 并替换 exe
            tracing::info!("开始安装 Windows 更新...");

            let current_exe = std::env::current_exe()?;
            let exe_dir = current_exe
                .parent()
                .ok_or_else(|| anyhow!("无法获取程序目录"))?;
            let backup_path = current_exe.with_extension("bak");
            let temp_extract_dir = exe_dir.join("update_temp");

            // 备份当前程序
            if current_exe.exists() {
                tracing::info!("备份当前程序到: {}", backup_path.display());
                fs::copy(&current_exe, &backup_path)?;
            }

            // 创建临时解压目录
            if temp_extract_dir.exists() {
                fs::remove_dir_all(&temp_extract_dir)?;
            }
            fs::create_dir_all(&temp_extract_dir)?;

            // 使用 PowerShell 解压 ZIP
            let output = std::process::Command::new("powershell")
                .arg("-Command")
                .arg(format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    download_path.display(),
                    temp_extract_dir.display()
                ))
                .output()?;

            if !output.status.success() {
                return Err(anyhow!("解压 ZIP 失败"));
            }

            // 查找解压后的 exe 文件
            let new_exe = temp_extract_dir.join("SteamCloudFileManager.exe");
            if !new_exe.exists() {
                return Err(anyhow!("未找到更新的程序文件"));
            }

            // 替换程序
            fs::copy(&new_exe, &current_exe)?;

            // 清理临时文件
            let _ = fs::remove_dir_all(&temp_extract_dir);
            let _ = fs::remove_file(download_path);

            tracing::info!("更新安装完成");
            Ok(())
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: 使用系统命令解压 tar.gz 并替换二进制
            tracing::info!("开始安装 Linux 更新...");

            let current_exe = std::env::current_exe()?;
            let exe_dir = current_exe
                .parent()
                .ok_or_else(|| anyhow!("无法获取程序目录"))?;
            let backup_path = current_exe.with_extension("bak");
            let temp_extract_dir = exe_dir.join("update_temp");

            // 备份当前程序
            if current_exe.exists() {
                tracing::info!("备份当前程序到: {}", backup_path.display());
                fs::copy(&current_exe, &backup_path)?;
            }

            // 创建临时解压目录
            if temp_extract_dir.exists() {
                fs::remove_dir_all(&temp_extract_dir)?;
            }
            fs::create_dir_all(&temp_extract_dir)?;

            // 使用 tar 命令解压
            let output = std::process::Command::new("tar")
                .arg("-xzf")
                .arg(download_path)
                .arg("-C")
                .arg(&temp_extract_dir)
                .output()?;

            if !output.status.success() {
                return Err(anyhow!("解压 tar.gz 失败"));
            }

            // 查找解压后的二进制文件
            let new_exe = temp_extract_dir.join("steam-cloud-file-manager");
            if !new_exe.exists() {
                return Err(anyhow!("未找到更新的程序文件"));
            }

            // 设置执行权限
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&new_exe)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&new_exe, perms)?;
            }

            // 替换程序
            fs::copy(&new_exe, &current_exe)?;

            // 清理临时文件
            let _ = fs::remove_dir_all(&temp_extract_dir);
            let _ = fs::remove_file(download_path);

            tracing::info!("更新安装完成");
            Ok(())
        }
    }

    // 重置状态
    pub fn reset(&mut self) {
        self.status = UpdateStatus::Idle;
    }

    // 设置错误状态
    pub fn set_error(&mut self, error: String) {
        self.status = UpdateStatus::Error(error);
    }

    // 打开 GitHub Release 页面
    pub fn open_release_page() {
        let url = format!("{}/releases", GITHUB_REPO_URL);
        if let Err(e) = open::that(&url) {
            tracing::error!("打开浏览器失败: {}", e);
        }
    }
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self::new()
    }
}
