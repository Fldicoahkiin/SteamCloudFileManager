use anyhow::{Result, anyhow};
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
    pub size: u64,
    pub timestamp: DateTime<Local>,
    pub is_persisted: bool,
    pub exists: bool,
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
        if let Ok(mut guard) = self.client.lock()
            && guard.is_some()
        {
            tracing::info!("断开 Steam 连接 (App ID: {})", self.app_id);
            *guard = None;
            drop(guard);
            unsafe {
                steamworks_sys::SteamAPI_Shutdown();
            }
            tracing::info!("Steam API 已关闭");

            // 等待 Steam 客户端处理断开
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        self.app_id = 0;
        Self::cleanup_app_id_file();
    }

    // Steam 会话当前登录账号的 32 位 account id（即 userdata 下的目录名）
    pub fn session_account_id(&self) -> Result<u32> {
        let guard = self
            .client
            .lock()
            .map_err(|_| anyhow!("Steam客户端锁不可用"))?;
        let client = guard.as_ref().ok_or_else(|| anyhow!("Steam客户端未连接"))?;
        Ok(client.user().steam_id().account_id().raw())
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
            root_description: crate::path_resolver::get_root_description(0),
        }
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
            .map_err(|e| anyhow!("读取文件 '{}' 失败: {}", filename, e))?;

        Ok(data)
    }

    pub fn write_file(&self, filename: &str, data: &[u8]) -> Result<bool> {
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;
        let remote_storage = client.remote_storage();

        // root 0 的本地副本就是 Steam 的写入目标。实测（App 480，配额不足）写入失败后
        // Steam 会删除这个本地文件，即使调用了 FileWriteStreamCancel；所以先备份，失败时写回。
        let account_id = client.user().steam_id().account_id().raw();
        let local_copy = crate::vdf_parser::VdfParser::find_steam_path()?
            .join("userdata")
            .join(account_id.to_string())
            .join(self.app_id.to_string())
            .join("remote")
            .join(filename);
        let backup =
            if local_copy.is_file() {
                Some(std::fs::read(&local_copy).map_err(|e| {
                    anyhow!("写入前无法备份本地文件 {}: {}", local_copy.display(), e)
                })?)
            } else {
                None
            };

        let written = write_cloud_file_atomically(filename, data);
        if written.is_err()
            && let Some(original) = &backup
        {
            restore_local_copy(&local_copy, original);
        }

        written.map_err(|e| {
            let size_str = crate::file_manager::format_size(data.len() as u64);

            // 检查云同步状态，辅助诊断
            let cloud_account = remote_storage.is_cloud_enabled_for_account();
            let cloud_app = remote_storage.is_cloud_enabled_for_app();

            let mut hints = Vec::new();
            if !cloud_account {
                hints.push("Steam 账户未启用云同步");
            }
            if !cloud_app {
                hints.push("该应用未启用云同步");
            }
            if hints.is_empty() {
                hints.push("可能原因: 云存储配额已满或文件名不合法");
            }

            anyhow!(
                "写入文件 '{}' 失败 (大小: {}, {}): {}",
                filename,
                size_str,
                hints.join(", "),
                e
            )
        })?;

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

    // 触发云同步
    pub fn sync_cloud_files(&self) -> Result<()> {
        let client = self.client.lock().unwrap();
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("Steam客户端未连接"))?;

        let remote_storage = client.remote_storage();

        // 检查云同步是否启用
        if !remote_storage.is_cloud_enabled_for_account() {
            return Err(anyhow!("账户未启用云同步"));
        }

        if !remote_storage.is_cloud_enabled_for_app() {
            return Err(anyhow!("应用未启用云同步"));
        }

        // 调用 run_callbacks 来触发同步
        // Steam API 会在后台自动同步文件
        client.run_callbacks();

        tracing::info!("已触发 Steam 云文件同步");
        Ok(())
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

// 写入失败后，本地文件丢失或内容变化时写回备份
fn restore_local_copy(path: &std::path::Path, original: &[u8]) {
    if std::fs::read(path).is_ok_and(|current| current == original) {
        return;
    }
    let restored = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|_| std::fs::write(path, original));
    match restored {
        Ok(()) => tracing::warn!("云端写入失败，已恢复本地文件: {}", path.display()),
        Err(e) => tracing::error!(
            "云端写入失败，恢复本地文件 {} 也失败: {}",
            path.display(),
            e
        ),
    }
}

// 分块写入云文件，任一块失败就取消写入流。
// steamworks 0.13 的 SteamFileWriter 在 Drop 时总是 FileWriteStreamClose，失败时会提交截断内容。
fn write_cloud_file_atomically(filename: &str, data: &[u8]) -> Result<()> {
    const CHUNK_SIZE: usize = 8 * 1024 * 1024;
    let name = std::ffi::CString::new(filename).map_err(|_| anyhow!("文件名包含 NUL 字符"))?;

    // SAFETY: 调用方持有已初始化的 Steam Client；name 在调用期间有效；
    // data 切片在每次 WriteChunk 期间有效，长度不超过 CHUNK_SIZE（远小于 i32::MAX）
    unsafe {
        let rs = steamworks_sys::SteamAPI_SteamRemoteStorage_v016();
        if rs.is_null() {
            return Err(anyhow!("ISteamRemoteStorage 不可用"));
        }
        let handle =
            steamworks_sys::SteamAPI_ISteamRemoteStorage_FileWriteStreamOpen(rs, name.as_ptr());
        if handle == steamworks_sys::k_UGCFileStreamHandleInvalid {
            return Err(anyhow!("无法打开云文件写入流"));
        }

        for chunk in data.chunks(CHUNK_SIZE) {
            let ok = steamworks_sys::SteamAPI_ISteamRemoteStorage_FileWriteStreamWriteChunk(
                rs,
                handle,
                chunk.as_ptr() as *const std::os::raw::c_void,
                chunk.len() as i32,
            );
            if !ok {
                steamworks_sys::SteamAPI_ISteamRemoteStorage_FileWriteStreamCancel(rs, handle);
                return Err(anyhow!("写入数据块失败，已取消写入"));
            }
        }

        if !steamworks_sys::SteamAPI_ISteamRemoteStorage_FileWriteStreamClose(rs, handle) {
            return Err(anyhow!("提交云文件写入失败"));
        }
    }
    Ok(())
}
