use crate::vdf_parser::VdfParser;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, TimeZone};
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use steamworks::Client;

#[derive(Default)]
pub struct SteamCloudManager {
    client: Arc<Mutex<Option<Client>>>,
    app_id: u32,
}

#[derive(Debug, Clone)]
pub struct CloudFile {
    pub name: String,
    pub size: i32,
    pub timestamp: DateTime<Local>,
    pub is_persisted: bool,
    pub exists: bool,
    #[allow(dead_code)]
    pub root: u32,
    pub root_description: String,
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
                log::warn!("无法创建 steam_appid.txt: {}", e);
            }

            // 尝试初始化
            let result = Client::init();

            // 清理文件
            Self::cleanup_app_id_file();

            if result.is_err() {
                // 如果失败，再尝试环境变量方式
                log::info!("steam_appid.txt方式失败，尝试环境变量初始化");
                Client::init()
            } else {
                result
            }
        };

        #[cfg(not(target_os = "windows"))]
        let init_result = {
            let mut result = Client::init();

            if result.is_err() {
                log::info!("环境变量初始化失败，尝试使用 steam_appid.txt");

                if let Err(e) = std::fs::write("steam_appid.txt", app_id.to_string()) {
                    log::warn!("无法创建 steam_appid.txt: {}", e);
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
                log::info!("断开 Steam 连接 (App ID: {})", self.app_id);
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

    pub fn get_files(&self) -> Result<Vec<CloudFile>> {
        // 首先尝试使用VDF解析器（能获取所有文件）
        if let Ok(vdf_files) = self.get_files_from_vdf() {
            if !vdf_files.is_empty() {
                log::info!("使用VDF解析器成功获取 {} 个文件", vdf_files.len());
                return Ok(vdf_files);
            }
        }

        // 如果VDF解析失败，回退到Steam API
        log::info!("VDF解析失败或无文件，尝试Steam API...");

        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        let remote_storage = client.remote_storage();

        // 检查云同步状态
        let cloud_enabled_account = remote_storage.is_cloud_enabled_for_account();
        let cloud_enabled_app = remote_storage.is_cloud_enabled_for_app();

        log::info!(
            "云同步状态 - 账户: {}, 应用: {}",
            cloud_enabled_account,
            cloud_enabled_app
        );

        if !cloud_enabled_account {
            log::warn!("此Steam账户未启用云同步功能");
        }

        if !cloud_enabled_app {
            log::warn!("此应用未启用云同步功能");
        }

        let steam_files = remote_storage.files();
        log::info!("Steam API返回 {} 个文件", steam_files.len());

        let mut files = Vec::new();

        for (i, steam_file) in steam_files.iter().enumerate() {
            log::debug!(
                "文件 {}: {} ({} bytes)",
                i + 1,
                steam_file.name,
                steam_file.size
            );

            let steam_file_handle = remote_storage.file(&steam_file.name);
            let timestamp = Local
                .timestamp_opt(steam_file_handle.timestamp(), 0)
                .single()
                .unwrap_or_else(Local::now);

            let file = CloudFile {
                name: steam_file.name.clone(),
                size: steam_file.size as i32,
                timestamp,
                is_persisted: steam_file_handle.is_persisted(),
                exists: steam_file_handle.exists(),
                root: 0,
                root_description: "Steam云文件夹 (remote)".to_string(),
            };
            files.push(file);
        }

        log::info!("最终返回 {} 个云文件", files.len());
        Ok(files)
    }

    /// 从remotecache.vdf获取文件列表（绕过Steam API限制）
    pub fn get_files_from_vdf(&self) -> Result<Vec<CloudFile>> {
        if self.app_id == 0 {
            return Err(anyhow!("未设置App ID"));
        }

        log::info!("尝试从VDF解析文件列表，App ID: {}", self.app_id);

        // 创建VDF解析器
        let parser = VdfParser::new()?;

        // 解析remotecache.vdf
        let vdf_entries = parser.parse_remotecache(self.app_id)?;

        let mut files = Vec::new();

        for entry in vdf_entries {
            log::debug!(
                "VDF文件: {} (root={}, size={}, 实际路径={:?})",
                entry.filename,
                entry.root,
                entry.size,
                entry.actual_path
            );

            let timestamp = Local
                .timestamp_opt(entry.timestamp, 0)
                .single()
                .unwrap_or_else(Local::now);

            let root_desc = Self::get_root_folder_name(entry.root);

            let cloud_file = CloudFile {
                name: entry.filename.clone(),
                size: entry.size,
                timestamp,
                is_persisted: entry.sync_state == 1,
                exists: entry
                    .actual_path
                    .as_ref()
                    .map(|p| p.exists())
                    .unwrap_or(false),
                root: entry.root,
                root_description: root_desc.clone(),
            };

            if let Some(path) = &entry.actual_path {
                log::info!(
                    "文件 {} 位于 {} ({})",
                    entry.filename,
                    root_desc,
                    path.display()
                );
            }

            files.push(cloud_file);
        }

        log::info!("VDF解析完成: {} 个文件", files.len());
        Ok(files)
    }

    pub fn get_quota(&self) -> Result<(u64, u64)> {
        // TODO: 升级到支持 GetQuota API 的 steamworks-rs 版本
        // Steam 原生 API ISteamRemoteStorage::GetQuota 可以获取准确配额
        // 当前 steamworks-rs 0.11 未暴露此接口，使用动态估算替代
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

    fn get_root_folder_name(root: u32) -> String {
        match root {
            0 => "Steam云文件夹",
            1 => "GameInstall",
            2 => {
                #[cfg(target_os = "windows")]
                {
                    "WinMyDocuments"
                }
                #[cfg(target_os = "macos")]
                {
                    "MacDocuments"
                }
                #[cfg(target_os = "linux")]
                {
                    "LinuxHome/Documents"
                }
            }
            3 => {
                #[cfg(target_os = "windows")]
                {
                    "WinAppDataRoaming"
                }
                #[cfg(target_os = "macos")]
                {
                    "MacApplicationSupport"
                }
                #[cfg(target_os = "linux")]
                {
                    "LinuxHome/.config"
                }
            }
            4 => {
                #[cfg(target_os = "windows")]
                {
                    "WinAppDataLocal"
                }
                #[cfg(target_os = "macos")]
                {
                    "MacCaches"
                }
                #[cfg(target_os = "linux")]
                {
                    "LinuxHome/.local/share"
                }
            }
            5 => {
                #[cfg(target_os = "macos")]
                {
                    "MacPreferences"
                }
                #[cfg(not(target_os = "macos"))]
                {
                    "Preferences"
                }
            }
            9 => {
                #[cfg(target_os = "windows")]
                {
                    "WinSavedGames"
                }
                #[cfg(not(target_os = "windows"))]
                {
                    "SavedGames"
                }
            }
            12 => {
                #[cfg(target_os = "windows")]
                {
                    "WinAppDataLocalLow"
                }
                #[cfg(not(target_os = "windows"))]
                {
                    "LocalLow"
                }
            }
            _ => "Unknown",
        }
        .to_string()
    }
}

impl Drop for SteamCloudManager {
    fn drop(&mut self) {
        if self.is_connected() {
            log::info!("关闭Steam API连接");
        }

        Self::cleanup_app_id_file();
    }
}
