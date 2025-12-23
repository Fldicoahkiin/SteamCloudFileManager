use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

const GITHUB_API_RELEASES: &str =
    "https://api.github.com/repos/Fldicoahkiin/SteamCloudFileManager/releases/latest";
const GITHUB_REPO_URL: &str = "https://github.com/Fldicoahkiin/SteamCloudFileManager";
const USER_AGENT: &str = "SteamCloudFileManager";
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 1000;
const DOWNLOAD_CHUNK_SIZE: usize = 8192; // 8KB chunks

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
    #[cfg(not(target_os = "macos"))]
    Installing,
    #[cfg(not(target_os = "macos"))]
    Success,
    Error(String),
}

// 更新管理器
pub struct UpdateManager {
    status: UpdateStatus,
    current_version: String,
    progress_rx: Option<Receiver<f32>>,
}

impl UpdateManager {
    pub fn new() -> Self {
        Self {
            status: UpdateStatus::Idle,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            progress_rx: None,
        }
    }

    pub fn status(&self) -> &UpdateStatus {
        &self.status
    }

    // 轮询下载进度
    pub fn poll_progress(&mut self) {
        if let Some(rx) = &self.progress_rx {
            let mut latest_progress = None;
            while let Ok(progress) = rx.try_recv() {
                latest_progress = Some(progress);
            }
            if let Some(progress) = latest_progress {
                self.status = UpdateStatus::Downloading(progress);
            }
        }
    }

    // 创建 HTTP agent，支持代理
    fn create_agent() -> ureq::Agent {
        use ureq::config::Config;

        let mut config_builder = Config::builder();

        // 从环境变量读取代理设置
        if let Ok(proxy_url) = std::env::var("HTTPS_PROXY")
            .or_else(|_| std::env::var("https_proxy"))
            .or_else(|_| std::env::var("HTTP_PROXY"))
            .or_else(|_| std::env::var("http_proxy"))
        {
            if !proxy_url.is_empty() {
                tracing::info!("使用代理: {}", proxy_url);
                if let Ok(proxy) = ureq::Proxy::new(&proxy_url) {
                    config_builder = config_builder.proxy(Some(proxy));
                }
            }
        }

        config_builder.build().new_agent()
    }

    // 带重试的 HTTP 请求
    fn request_with_retry<F, T>(operation: F) -> Result<T>
    where
        F: Fn() -> Result<T>,
    {
        let mut last_error = anyhow!("未知错误");

        for attempt in 1..=MAX_RETRIES {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = e;
                    if attempt < MAX_RETRIES {
                        tracing::warn!(
                            "请求失败 (尝试 {}/{}): {}",
                            attempt,
                            MAX_RETRIES,
                            last_error
                        );
                        std::thread::sleep(std::time::Duration::from_millis(
                            RETRY_DELAY_MS * attempt as u64,
                        ));
                    }
                }
            }
        }

        Err(last_error)
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
        Self::request_with_retry(|| {
            let agent = Self::create_agent();
            let response = agent
                .get(GITHUB_API_RELEASES)
                .header("User-Agent", USER_AGENT)
                .header("Accept", "application/vnd.github.v3+json")
                .call()
                .map_err(|e| anyhow!("请求失败: {}", e))?;

            let release: ReleaseInfo = serde_json::from_reader(response.into_body().as_reader())
                .map_err(|e| anyhow!("解析响应失败: {}", e))?;

            Ok(release)
        })
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

        let (result_tx, result_rx) = channel();
        let (progress_tx, progress_rx) = channel();
        let release_clone = release.clone();

        // 保存进度接收器
        self.progress_rx = Some(progress_rx);

        std::thread::spawn(move || {
            let result = Self::download_in_background(&release_clone, progress_tx);
            let _ = result_tx.send(result);
        });

        self.status = UpdateStatus::Downloading(0.0);
        result_rx
    }

    // 后台下载
    fn download_in_background(
        release: &ReleaseInfo,
        progress_tx: Sender<f32>,
    ) -> Result<PathBuf, String> {
        // 根据平台选择对应的资源文件
        let asset = Self::select_asset_for_platform(&release.assets).map_err(|e| e.to_string())?;
        tracing::info!("选择资源文件: {}", asset.name);

        // 下载文件
        let update_dir = Self::get_update_dir().map_err(|e| e.to_string())?;
        let download_path = update_dir.join(&asset.name);

        tracing::info!("下载文件: {}", asset.name);
        tracing::info!("下载地址: {}", asset.browser_download_url);
        tracing::info!("保存路径: {}", download_path.display());
        tracing::info!("文件大小: {:.2} MB", asset.size as f64 / 1024.0 / 1024.0);

        tracing::info!("开始下载...");

        // 如果文件已存在，先删除
        if download_path.exists() {
            tracing::info!("删除已存在的旧文件");
            fs::remove_file(&download_path).map_err(|e| e.to_string())?;
        }

        // 发送初始进度
        let _ = progress_tx.send(0.0);

        let agent = Self::create_agent();
        let response = agent
            .get(&asset.browser_download_url)
            .header("User-Agent", USER_AGENT)
            .call()
            .map_err(|e| {
                tracing::error!("HTTP 请求失败: {}", e);
                format!("下载失败: {}", e)
            })?;

        let mut file = fs::File::create(&download_path).map_err(|e| e.to_string())?;
        let mut body = response.into_body();
        let mut reader = body.as_reader();

        // 分块下载并报告进度
        let total_size = asset.size;
        let mut downloaded: u64 = 0;
        let mut buffer = vec![0u8; DOWNLOAD_CHUNK_SIZE];
        let mut last_progress_percent = 0;

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    use std::io::Write;
                    file.write_all(&buffer[..n]).map_err(|e| {
                        tracing::error!("写入文件失败: {}", e);
                        format!("写入文件失败: {}", e)
                    })?;

                    downloaded += n as u64;
                    let progress = if total_size > 0 {
                        (downloaded as f32 / total_size as f32).min(1.0)
                    } else {
                        0.0
                    };

                    // 每 1% 发送一次进度更新
                    let progress_percent = (progress * 100.0) as i32;
                    if progress_percent > last_progress_percent {
                        last_progress_percent = progress_percent;
                        let _ = progress_tx.send(progress);
                        tracing::debug!("下载进度: {:.1}%", progress * 100.0);
                    }
                }
                Err(e) => {
                    tracing::error!("读取数据失败: {}", e);
                    return Err(format!("读取数据失败: {}", e));
                }
            }
        }

        // 发送完成进度
        let _ = progress_tx.send(1.0);

        tracing::info!("下载完成: {}", download_path.display());
        Ok(download_path)
    }

    // 安装已下载的更新 (Windows/Linux 下使用)
    #[cfg(not(target_os = "macos"))]
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

    // 安装更新 (Windows/Linux 下使用)
    #[cfg(not(target_os = "macos"))]
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
            // Windows: 使用批处理脚本延迟替换 exe
            tracing::info!("开始安装 Windows 更新...");

            let current_exe = std::env::current_exe()?;
            let exe_dir = current_exe
                .parent()
                .ok_or_else(|| anyhow!("无法获取程序目录"))?;
            let temp_extract_dir = exe_dir.join("update_temp");
            let update_script = exe_dir.join("update.bat");

            // 创建临时解压目录
            if temp_extract_dir.exists() {
                fs::remove_dir_all(&temp_extract_dir)?;
            }
            fs::create_dir_all(&temp_extract_dir)?;

            // 使用 PowerShell 解压 ZIP
            tracing::info!("解压更新包...");
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

            // 创建批处理脚本来延迟替换
            let batch_content = format!(
                concat!(
                    "@echo off\n",
                    "chcp 65001 >nul\n",
                    "echo 正在更新 Steam Cloud File Manager...\n",
                    "echo.\n",
                    "echo 等待程序退出...\n",
                    "timeout /t 2 /nobreak >nul\n",
                    ":wait_loop\n",
                    "tasklist /FI \"IMAGENAME eq SteamCloudFileManager.exe\" 2>NUL | find /I /N \"SteamCloudFileManager.exe\">NUL\n",
                    "if \"%ERRORLEVEL%\"==\"0\" (\n",
                    "    timeout /t 1 /nobreak >nul\n",
                    "    goto wait_loop\n",
                    ")\n",
                    "echo 程序已退出，开始更新...\n",
                    "copy /Y \"{}\" \"{}\"\n",
                    "if errorlevel 1 (\n",
                    "    echo 更新失败！请手动替换程序文件。\n",
                    "    pause\n",
                    "    exit /b 1\n",
                    ")\n",
                    "echo 更新完成！正在启动程序...\n",
                    "rmdir /S /Q \"{}\"\n",
                    "del \"{}\"\n",
                    "start \"\" /B \"{}\"\n",
                    "del \"%~f0\"\n",
                ),
                new_exe.display(),
                current_exe.display(),
                temp_extract_dir.display(),
                download_path.display(),
                current_exe.display()
            );

            fs::write(&update_script, batch_content)?;
            tracing::info!("创建更新脚本: {}", update_script.display());

            // 启动批处理脚本并退出当前程序
            tracing::info!("启动更新脚本并退出程序...");
            std::process::Command::new("cmd")
                .args(["/C", "start", "", "/MIN", &update_script.to_string_lossy()])
                .spawn()?;

            // 退出当前程序
            std::process::exit(0);
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
