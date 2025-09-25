use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, TimeZone};
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use steamworks::{Client, SingleClient};

#[derive(Default)]
pub struct SteamCloudManager {
    client: Arc<Mutex<Option<Client>>>,
    single_client: Arc<Mutex<Option<SingleClient>>>,
    app_id: u32,
}

#[derive(Debug, Clone)]
pub struct CloudFile {
    pub name: String,
    pub size: i32,
    pub timestamp: DateTime<Local>,
    pub is_persisted: bool,
    pub exists: bool,
}

impl SteamCloudManager {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            single_client: Arc::new(Mutex::new(None)),
            app_id: 0,
        }
    }

    fn cleanup_app_id_file() {
        if Path::new("steam_appid.txt").exists() {
            let _ = std::fs::remove_file("steam_appid.txt");
        }
    }

    pub fn connect(&mut self, app_id: u32) -> Result<()> {
        // 如果已经连接，先断开之前的连接
        self.disconnect();

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

        let (client, single) = init_result.map_err(|e| anyhow!("无法初始化Steam API: {:?}", e))?;

        if let Ok(mut guard) = self.client.lock() {
            *guard = Some(client);
        } else {
            return Err(anyhow!("Steam客户端锁不可用"));
        }
        if let Ok(mut guard) = self.single_client.lock() {
            *guard = Some(single);
        } else {
            return Err(anyhow!("Steam SingleClient 锁不可用"));
        }
        self.app_id = app_id;

        Ok(())
    }

    pub fn disconnect(&mut self) {
        if let Ok(mut guard) = self.client.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = self.single_client.lock() {
            *guard = None;
        }
        self.app_id = 0;
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
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        let remote_storage = client.remote_storage();
        let steam_files = remote_storage.files();
        let mut files = Vec::new();

        for steam_file in steam_files {
            let steam_file_handle = remote_storage.file(&steam_file.name);
            let timestamp = Local
                .timestamp_opt(steam_file_handle.timestamp(), 0)
                .single()
                .unwrap_or_else(|| Local::now());

            let file = CloudFile {
                name: steam_file.name,
                size: steam_file.size as i32,
                timestamp,
                is_persisted: steam_file_handle.is_persisted(),
                exists: steam_file_handle.exists(),
            };
            files.push(file);
        }

        Ok(files)
    }

    pub fn get_quota(&self) -> Result<(u64, u64)> {
        // TODO: 升级到支持 GetQuota API 的 steamworks-rs 版本
        // Steam 原生 API ISteamRemoteStorage::GetQuota 可以获取准确配额
        // 当前 steamworks-rs 0.11 未暴露此接口，使用动态估算替代
        let used_bytes = self.calculate_used_space()?;

        // 根据已用空间动态估算总配额
        // Steam 云存储配额通常是 100MB、200MB、1GB 等固定值
        let estimated_total = if used_bytes == 0 {
            100_000_000u64 // 默认100MB（新游戏常见配额）
        } else if used_bytes < 50_000_000 {
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
        if let Ok(guard) = self.single_client.lock() {
            if let Some(single) = guard.as_ref() {
                single.run_callbacks();
            }
        }
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
