use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
    Installing,
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

    // 下载并安装更新
    pub fn download_and_install(&mut self, release: &ReleaseInfo) -> Result<()> {
        tracing::info!("开始下载更新: {}", release.tag_name);

        // 根据平台选择对应的资源文件
        let asset = Self::select_asset_for_platform(&release.assets)?;
        tracing::info!("选择资源文件: {}", asset.name);

        // 下载文件
        let download_path = self.download_asset(&asset)?;
        tracing::info!("下载完成: {}", download_path.display());

        // 安装更新
        self.status = UpdateStatus::Installing;
        self.install_update(&download_path)?;

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
            ("windows", "x86_64") => "windows-x86_64.exe",
            ("linux", "x86_64") => "linux-x86_64",
            _ => return Err(anyhow!("不支持的平台: {} ({})", platform, arch)),
        };

        assets
            .iter()
            .find(|a| a.name.contains(pattern))
            .cloned()
            .ok_or_else(|| anyhow!("未找到适合当前平台的安装包"))
    }

    // 下载资源文件
    fn download_asset(&mut self, asset: &ReleaseAsset) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let download_path = temp_dir.join(&asset.name);

        tracing::info!("下载到: {}", download_path.display());

        let response = ureq::get(&asset.browser_download_url)
            .call()
            .map_err(|e| anyhow!("下载失败: {}", e))?;

        let total_size = asset.size;
        let mut downloaded: u64 = 0;
        let mut file = fs::File::create(&download_path)?;

        let mut reader = response.into_body().into_reader();
        let mut buffer = [0; 8192];

        loop {
            let n = std::io::Read::read(&mut reader, &mut buffer)?;
            if n == 0 {
                break;
            }
            std::io::Write::write_all(&mut file, &buffer[..n])?;
            downloaded += n as u64;

            let progress = downloaded as f32 / total_size as f32;
            self.status = UpdateStatus::Downloading(progress);
        }

        Ok(download_path)
    }

    // 安装更新
    fn install_update(&self, download_path: &PathBuf) -> Result<()> {
        let current_exe = std::env::current_exe()?;
        let backup_path = current_exe.with_extension("bak");

        tracing::info!("备份当前程序到: {}", backup_path.display());

        // 备份当前程序
        if current_exe.exists() {
            fs::copy(&current_exe, &backup_path)?;
        }

        // 替换程序
        #[cfg(target_os = "macos")]
        {
            // macOS 需要设置执行权限
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(download_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(download_path, perms)?;
        }

        #[cfg(target_os = "linux")]
        {
            // Linux 需要设置执行权限
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(download_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(download_path, perms)?;
        }

        // 替换文件
        fs::copy(download_path, &current_exe)?;

        // 清理下载文件
        let _ = fs::remove_file(download_path);

        tracing::info!("更新安装完成，请重启应用");
        Ok(())
    }

    // 重置状态
    pub fn reset(&mut self) {
        self.status = UpdateStatus::Idle;
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
