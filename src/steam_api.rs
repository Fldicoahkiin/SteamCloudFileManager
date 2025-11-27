use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, TimeZone};
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use steamworks::Client;

pub fn restart_steam_with_debugging() -> Result<()> {
    tracing::info!("正在尝试以调试模式重启 Steam...");

    #[cfg(target_os = "macos")]
    {
        // 关闭现有 Steam
        let _ = Command::new("pkill").arg("-f").arg("Steam").status();
        let _ = Command::new("pkill").arg("-f").arg("steam_osx").status();

        std::thread::sleep(std::time::Duration::from_secs(2));

        // open -a Steam --args -cef-enable-debugging
        Command::new("open")
            .arg("-a")
            .arg("Steam")
            .arg("--args")
            .arg("-cef-enable-debugging")
            .spawn()
            .map_err(|e| anyhow!("无法启动 Steam: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        // 关闭现有 Steam
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "steam.exe"])
            .status();

        std::thread::sleep(std::time::Duration::from_secs(2));

        // 找到并启动 Steam
        let steam_dir = crate::vdf_parser::VdfParser::find_steam_path()?;
        let steam_exe = steam_dir.join("steam.exe");

        if !steam_exe.exists() {
            return Err(anyhow!("找不到 steam.exe"));
        }

        // 使用 cmd /C start 来启动
        Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(steam_exe)
            .arg("-cef-enable-debugging")
            .spawn()
            .map_err(|e| anyhow!("无法启动 Steam: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("pkill").arg("steam").status();

        std::thread::sleep(std::time::Duration::from_secs(2));

        Command::new("steam")
            .arg("-cef-enable-debugging")
            .spawn()
            .or_else(|_| {
                // 如果 PATH 中没有，尝试使用 VdfParser 找到的路径
                let _steam_dir = crate::vdf_parser::VdfParser::find_steam_path()?;
                // Linux 下可能是 steam.sh 或者 ubuntu 下是 /usr/games/steam
                Err(anyhow!("无法启动 Steam (Linux)"))
            })?;
    }

    Ok(())
}

#[derive(Default)]
pub struct SteamCloudManager {
    client: Arc<Mutex<Option<Client>>>,
    app_id: u32,
}

#[derive(Debug, Clone)]
pub struct CloudFile {
    pub name: String,
    pub size: u64,
    pub timestamp: DateTime<Local>,
    pub is_persisted: bool,
    pub exists: bool,
    pub root: u32,
    pub root_description: String,
    #[allow(dead_code)]
    pub conflict: bool,
}

impl SteamCloudManager {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            app_id: 0,
        }
    }

    fn cleanup_app_id_file() {
        if Path::new("steam_appid.txt").exists() {
            let _ = std::fs::remove_file("steam_appid.txt");
        }
    }

    pub fn connect(&mut self, app_id: u32) -> Result<()> {
        if self.is_connected() {
            self.disconnect();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        Self::cleanup_app_id_file();

        #[cfg(target_os = "windows")]
        {
            unsafe {
                std::env::set_var("SteamAppId", app_id.to_string());
                std::env::set_var("SteamAppID", app_id.to_string());
                std::env::set_var("SteamGameId", app_id.to_string());
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            unsafe {
                std::env::set_var("SteamAppId", app_id.to_string());
                std::env::set_var("SteamGameId", app_id.to_string());
            }
        }

        #[cfg(target_os = "windows")]
        let init_result = {
            // 创建steam_appid.txt文件
            if let Err(e) = std::fs::write("steam_appid.txt", app_id.to_string()) {
                tracing::warn!("无法创建 steam_appid.txt: {}", e);
            }

            // 尝试初始化
            let result = Client::init();

            // 清理文件
            Self::cleanup_app_id_file();

            if result.is_err() {
                // 如果失败，再尝试环境变量方式
                tracing::info!("steam_appid.txt方式失败，尝试环境变量初始化");
                Client::init()
            } else {
                result
            }
        };

        #[cfg(not(target_os = "windows"))]
        let init_result = {
            let mut result = Client::init();

            if result.is_err() {
                tracing::info!("环境变量初始化失败，尝试使用 steam_appid.txt");

                if let Err(e) = std::fs::write("steam_appid.txt", app_id.to_string()) {
                    tracing::warn!("无法创建 steam_appid.txt: {}", e);
                }

                result = Client::init();
                Self::cleanup_app_id_file();
            }
            result
        };

        let client = init_result.map_err(|e| anyhow!("无法初始化Steam API: {:?}", e))?;

        if let Ok(mut guard) = self.client.lock() {
            *guard = Some(client);
        } else {
            return Err(anyhow!("Steam客户端锁不可用"));
        }
        self.app_id = app_id;

        Ok(())
    }

    pub fn disconnect(&mut self) {
        if let Ok(mut guard) = self.client.lock() {
            if guard.is_some() {
                tracing::info!("断开 Steam 连接 (App ID: {})", self.app_id);
                *guard = None;
                drop(guard);
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        self.app_id = 0;
        Self::cleanup_app_id_file();
    }

    pub fn is_connected(&self) -> bool {
        match self.client.lock() {
            Ok(guard) => guard.is_some(),
            Err(_poison) => false,
        }
    }

    pub fn is_cloud_enabled_for_account(&self) -> Result<bool> {
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        Ok(client.remote_storage().is_cloud_enabled_for_account())
    }

    pub fn is_cloud_enabled_for_app(&self) -> Result<bool> {
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        Ok(client.remote_storage().is_cloud_enabled_for_app())
    }

    pub fn set_cloud_enabled_for_app(&self, enabled: bool) -> Result<()> {
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        client.remote_storage().set_cloud_enabled_for_app(enabled);
        Ok(())
    }

    // 从 Steam API 获取文件列表
    // 注意：这个方法现在由 FileService 统一调用
    pub fn get_files_from_api(&self) -> Result<Vec<CloudFile>> {
        let client = self.client.lock().unwrap();
        let client = client.as_ref().ok_or_else(|| anyhow!("未连接到 Steam"))?;

        let remote_storage = client.remote_storage();

        // 检查云同步状态
        let cloud_enabled_account = remote_storage.is_cloud_enabled_for_account();
        let cloud_enabled_app = remote_storage.is_cloud_enabled_for_app();

        tracing::debug!(
            "云同步状态 - 账户: {}, 应用: {}",
            cloud_enabled_account,
            cloud_enabled_app
        );

        if !cloud_enabled_account {
            tracing::info!("此 Steam 账户未启用云同步功能");
        }

        if !cloud_enabled_app {
            tracing::info!("此应用未启用云同步功能");
        }

        let steam_files = remote_storage.files();
        tracing::debug!("Steam API 返回 {} 个文件", steam_files.len());

        let files: Vec<CloudFile> = steam_files
            .iter()
            .map(|steam_file| {
                let file_handle = remote_storage.file(&steam_file.name);
                self.build_cloud_file_from_api(&file_handle, &steam_file.name, steam_file.size)
            })
            .collect();

        tracing::debug!("构建完成 {} 个 CloudFile 对象", files.len());
        Ok(files)
    }

    // 从 Steam API 构建 CloudFile
    fn build_cloud_file_from_api(
        &self,
        file_handle: &steamworks::SteamFile,
        name: &str,
        size: u64,
    ) -> CloudFile {
        let timestamp = Local
            .timestamp_opt(file_handle.timestamp(), 0)
            .single()
            .unwrap_or_else(Local::now);

        CloudFile {
            name: name.to_string(),
            size,
            timestamp,
            is_persisted: file_handle.is_persisted(),
            exists: file_handle.exists(),
            root: 0,
            root_description: "Steam云文件夹 (Remote)".to_string(),
            conflict: false,
        }
    }

    pub fn get_quota(&self) -> Result<(u64, u64)> {
        // 尝试使用 unsafe 直接调用底层 API
        unsafe {
            use steamworks_sys as sys;

            // 获取 ISteamRemoteStorage 接口指针 (SDK 1.57+ 对应 v016)
            let interface = sys::SteamAPI_SteamRemoteStorage_v016();

            if !interface.is_null() {
                let mut total: u64 = 0;
                let mut available: u64 = 0;
                if sys::SteamAPI_ISteamRemoteStorage_GetQuota(interface, &mut total, &mut available)
                {
                    tracing::debug!(
                        "通过 unsafe API 获取配额: 总计 {} / 可用 {}",
                        total,
                        available
                    );
                    return Ok((total, available));
                }
            }
        }

        // 回退到动态估算
        tracing::debug!("无法通过 unsafe API 获取配额，使用估算值");
        let used_bytes = self.calculate_used_space()?;

        // 根据已用空间动态估算总配额
        // Steam 云存储配额通常是 100MB、200MB、1GB 等固定值
        let estimated_total = if used_bytes < 50_000_000 {
            100_000_000u64 // < 50MB，可能是100MB配额
        } else if used_bytes < 100_000_000 {
            200_000_000u64 // 50-100MB，可能是200MB配额
        } else if used_bytes < 500_000_000 {
            1_000_000_000u64 // 100-500MB，可能是1GB配额
        } else {
            // 超过500MB，按已用空间1.5倍估算
            (used_bytes as f64 * 1.5) as u64
        };

        let available_bytes = estimated_total.saturating_sub(used_bytes);

        tracing::debug!(
            "配额估算: 已用 {} / 估算总量 {} (可用 {})",
            used_bytes,
            estimated_total,
            available_bytes
        );
        Ok((estimated_total, available_bytes))
    }

    fn calculate_used_space(&self) -> Result<u64> {
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        let remote_storage = client.remote_storage();
        let files = remote_storage.files();

        let total_size: u64 = files.iter().map(|f| f.size).sum();
        Ok(total_size)
    }

    pub fn read_file(&self, filename: &str) -> Result<Vec<u8>> {
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        let remote_storage = client.remote_storage();
        let file_handle = remote_storage.file(filename);

        if !file_handle.exists() {
            return Err(anyhow!("文件不存在: {}", filename));
        }

        let mut reader = file_handle.read();
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .map_err(|e| anyhow!("读取文件失败: {}", e))?;

        Ok(data)
    }

    pub fn write_file(&self, filename: &str, data: &[u8]) -> Result<bool> {
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        let remote_storage = client.remote_storage();
        let file_handle = remote_storage.file(filename);

        use std::io::Write;
        let mut writer = file_handle.write();
        writer
            .write_all(data)
            .map_err(|e| anyhow!("写入文件失败: {}", e))?;

        Ok(true)
    }

    pub fn delete_file(&self, filename: &str) -> Result<bool> {
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        let remote_storage = client.remote_storage();
        let file_handle = remote_storage.file(filename);

        Ok(file_handle.delete())
    }

    pub fn forget_file(&self, filename: &str) -> Result<bool> {
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        let remote_storage = client.remote_storage();
        let file_handle = remote_storage.file(filename);

        Ok(file_handle.forget())
    }

    pub fn run_callbacks(&self) {
        if let Ok(guard) = self.client.lock() {
            if let Some(client) = guard.as_ref() {
                client.run_callbacks();
            }
        }
    }
}

impl Drop for SteamCloudManager {
    fn drop(&mut self) {
        if self.is_connected() {
            tracing::info!("关闭Steam API连接");
        }

        Self::cleanup_app_id_file();
    }
}
