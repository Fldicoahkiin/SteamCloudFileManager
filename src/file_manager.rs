use crate::path_resolver::{get_root_description, resolve_cloud_file_path};
use crate::steam_api::CloudFile;
use crate::vdf_parser::{VdfFileEntry, VdfParser};
use anyhow::{anyhow, Result};
use chrono::{Local, TimeZone};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct FileService {
    steam_manager: Option<Arc<Mutex<crate::steam_api::SteamCloudManager>>>,
}

impl FileService {
    pub fn new() -> Self {
        Self {
            steam_manager: None,
        }
    }

    pub fn with_steam_manager(
        steam_manager: Arc<Mutex<crate::steam_api::SteamCloudManager>>,
    ) -> Self {
        Self {
            steam_manager: Some(steam_manager),
        }
    }

    // 获取云文件列表
    // VDF -> Steam API 回退 -> CDP 补充
    pub fn get_cloud_files(&self, app_id: u32) -> Result<Vec<CloudFile>> {
        if app_id == 0 {
            return Err(anyhow!("未设置 App ID"));
        }

        // VDF
        let vdf_files = self.get_files_from_vdf(app_id)?;
        if !vdf_files.is_empty() {
            tracing::info!("获取到 {} 个云文件", vdf_files.len());
            return Ok(vdf_files);
        } else {
            tracing::debug!("VDF 返回空列表，尝试其他方式");
        }

        // Steam API
        if let Some(manager) = &self.steam_manager {
            match self.get_files_from_steam_api(manager) {
                Ok(files) => {
                    tracing::info!(source = "Steam API", "获取云文件成功");
                    return Ok(files);
                }
                Err(e) => tracing::warn!(error = %e, "Steam API 获取失败"),
            }
        }

        Err(anyhow!("无法获取文件列表：所有策略均失败"))
    }

    // 从 VDF 获取文件列表
    fn get_files_from_vdf(&self, app_id: u32) -> Result<Vec<CloudFile>> {
        tracing::debug!(app_id = app_id, "尝试从 VDF 解析文件列表");

        let parser = VdfParser::new()?;
        let vdf_entries = parser.parse_remotecache(app_id)?;

        let steam_path = parser.get_steam_path().clone();
        let user_id = parser.get_user_id().to_string();

        let files: Vec<CloudFile> = vdf_entries
            .into_iter()
            .map(|entry| build_cloud_file_from_vdf(entry, &steam_path, &user_id, app_id))
            .collect();

        tracing::debug!(count = files.len(), "VDF 解析完成");
        Ok(files)
    }

    // 从 Steam API 获取文件列表
    fn get_files_from_steam_api(
        &self,
        manager: &Arc<Mutex<crate::steam_api::SteamCloudManager>>,
    ) -> Result<Vec<CloudFile>> {
        let mgr = manager.lock().map_err(|e| anyhow!("锁错误: {}", e))?;
        mgr.get_files_from_api()
    }

    // 合并 CDP 数据到现有文件列表
    pub fn merge_cdp_files(
        &self,
        mut files: Vec<CloudFile>,
        app_id: u32,
    ) -> Result<Vec<CloudFile>> {
        if !crate::cdp_client::CdpClient::is_cdp_running() {
            return Ok(files);
        }

        tracing::info!("尝试通过 CDP 补充文件信息");

        if let Ok(mut client) = crate::cdp_client::CdpClient::connect() {
            if let Ok(cdp_files) = client.fetch_game_files(app_id) {
                tracing::info!(count = cdp_files.len(), "CDP 返回文件");

                let file_map: std::collections::HashMap<String, usize> = files
                    .iter()
                    .enumerate()
                    .map(|(i, f)| (f.name.clone(), i))
                    .collect();

                for cdp_file in cdp_files {
                    if let Some(&idx) = file_map.get(&cdp_file.name) {
                        let f = &mut files[idx];
                        let vdf_root_desc = f.root_description.clone();

                        f.size = cdp_file.size;
                        f.timestamp = cdp_file.timestamp;
                        f.is_persisted = true;

                        if cdp_file.root_description.starts_with("CDP:") {
                            // 提取 URL 和 CDP 文件夹名
                            let content = &cdp_file.root_description[4..];
                            let parts: Vec<&str> = content.split('|').collect();
                            let url = parts.first().unwrap_or(&"");
                            let cdp_folder = parts.get(1).unwrap_or(&"");

                            f.root_description = cdp_file.root_description.clone();

                            tracing::debug!(
                                "合并 CDP 文件: {} | Root={} | 本地存在={} | VDF: {} | CDP: {} | URL: {}",
                                f.name,
                                f.root,
                                f.exists,
                                vdf_root_desc,
                                cdp_folder,
                                url
                            );
                        } else {
                            tracing::debug!(
                                "合并 CDP 文件: {} | Root={} | 本地存在={} | VDF: {} | 保留原 root_description",
                                f.name,
                                f.root,
                                f.exists,
                                f.root_description
                            );
                        }
                    } else {
                        tracing::debug!(
                            "新增 CDP 文件: {} | Root={} | 本地存在={} | {}",
                            cdp_file.name,
                            cdp_file.root,
                            cdp_file.exists,
                            cdp_file.root_description
                        );
                        files.push(cdp_file);
                    }
                }
            }
        }

        Ok(files)
    }
}

impl Default for FileService {
    fn default() -> Self {
        Self::new()
    }
}

// 从 VDF 条目构建 CloudFile
fn build_cloud_file_from_vdf(
    entry: VdfFileEntry,
    steam_path: &std::path::Path,
    user_id: &str,
    app_id: u32,
) -> CloudFile {
    let timestamp = if entry.timestamp > 0 {
        Local
            .timestamp_opt(entry.timestamp, 0)
            .single()
            .unwrap_or_else(Local::now)
    } else {
        Local::now()
    };

    let root_desc = get_root_description(entry.root);

    // 使用 path_resolver 解析路径
    let actual_path =
        resolve_cloud_file_path(entry.root, &entry.filename, steam_path, user_id, app_id).ok();

    let exists = actual_path.as_ref().map(|p| p.exists()).unwrap_or(false);

    CloudFile {
        name: entry.filename,
        size: entry.size,
        timestamp,
        is_persisted: entry.sync_state == 2,
        root: entry.root,
        root_description: root_desc,
        exists,
        conflict: false,
    }
}

// 文件操作结构体
pub struct FileOperations {
    steam_manager: Arc<Mutex<crate::steam_api::SteamCloudManager>>,
}

impl FileOperations {
    pub fn new(steam_manager: Arc<Mutex<crate::steam_api::SteamCloudManager>>) -> Self {
        Self { steam_manager }
    }

    // 下载文件到指定路径
    pub fn download_file(&self, file: &CloudFile, path: &PathBuf) -> Result<()> {
        tracing::debug!(
            "尝试下载文件: {}, root_desc: {}",
            file.name,
            if file.root_description.starts_with("CDP:") {
                "CDP URL"
            } else {
                &file.root_description
            }
        );

        let url_prefix = "CDP:";

        // 优先尝试 CDP 下载
        if file.root_description.starts_with(url_prefix) {
            let content = &file.root_description[url_prefix.len()..];
            let url = content.split('|').next().unwrap_or("");

            if !url.is_empty() {
                tracing::debug!("使用 CDP 下载: {} -> {}", file.name, url);

                match ureq::get(url).call() {
                    Ok(resp) => {
                        let mut reader = resp.into_body().into_reader();
                        let mut data = Vec::new();
                        std::io::Read::read_to_end(&mut reader, &mut data)
                            .map_err(|e| anyhow!("CDP 读取响应流失败: {}", e))?;

                        self.save_data_to_path(&data, path)?;
                        return Ok(());
                    }
                    Err(e) => {
                        tracing::warn!("CDP 下载失败，尝试 Steam API: {}", e);
                    }
                }
            }
        }

        // 如果 CDP 下载失败或不可用，尝试 Steam API
        tracing::debug!("使用 Steam API 下载: {}", file.name);
        let manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;

        let data = manager
            .read_file(&file.name)
            .map_err(|e| anyhow!("Steam API 下载失败: {}", e))?;

        self.save_data_to_path(&data, path)?;
        Ok(())
    }

    // 删除文件
    pub fn delete_file(&self, filename: &str) -> Result<bool> {
        let manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;

        manager
            .delete_file(filename)
            .map_err(|e| anyhow!("删除文件失败: {}", e))
    }

    // 取消云同步
    pub fn forget_file(&self, filename: &str) -> anyhow::Result<bool> {
        let manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;
        manager.forget_file(filename)
    }

    // 保存数据到路径
    fn save_data_to_path(&self, data: &[u8], path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| anyhow!("创建目录失败: {}", e))?;
            }
        }

        std::fs::write(path, data).map_err(|e| anyhow!("保存文件失败: {}", e))?;

        tracing::info!("文件已保存: {}", path.display());
        Ok(())
    }

    pub fn batch_operation<F>(&self, filenames: &[String], operation: F) -> (usize, Vec<String>)
    where
        F: Fn(&str) -> anyhow::Result<bool>,
    {
        let mut success_count = 0;
        let mut failed_files = Vec::new();

        for filename in filenames {
            match operation(filename) {
                Ok(true) => success_count += 1,
                Ok(false) => failed_files.push(filename.to_string()),
                Err(e) => failed_files.push(format!("{} (错误: {})", filename, e)),
            }
        }

        (success_count, failed_files)
    }
}

// 批量下载文件（保持文件夹结构）
pub fn batch_download_files_with_dialog(
    files: &[CloudFile],
    selected_indices: &[usize],
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> Result<Option<(usize, Vec<String>)>> {
    use rfd::FileDialog;

    if selected_indices.is_empty() {
        return Err(anyhow!("请选择要下载的文件"));
    }

    // 选择保存目录
    if let Some(base_dir) = FileDialog::new().pick_folder() {
        let file_ops = FileOperations::new(steam_manager);
        let mut success_count = 0;
        let mut failed_files = Vec::new();

        for &index in selected_indices {
            if index >= files.len() {
                continue;
            }

            let file = &files[index];
            // 保持文件夹结构：使用文件的原始名称（包含路径）
            let file_path = base_dir.join(&file.name);

            match file_ops.download_file(file, &file_path) {
                Ok(_) => success_count += 1,
                Err(e) => failed_files.push(format!("{}: {}", file.name, e)),
            }
        }

        Ok(Some((success_count, failed_files)))
    } else {
        Ok(None)
    }
}

// 文件操作结果
pub enum FileOperationResult {
    Error(String),              // 错误消息
    SuccessWithRefresh(String), // 成功消息 + 需要刷新
}

// 批量取消云同步
pub fn batch_forget_files(
    filenames: &[String],
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> (usize, Vec<String>) {
    let file_ops = FileOperations::new(steam_manager);
    file_ops.batch_operation(filenames, |filename| file_ops.forget_file(filename))
}

// 批量删除文件
pub fn batch_delete_files(
    filenames: &[String],
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> (usize, Vec<String>) {
    let file_ops = FileOperations::new(steam_manager);
    file_ops.batch_operation(filenames, |filename| file_ops.delete_file(filename))
}

// 取消云同步选中的文件
pub fn forget_selected_files_coordinated(
    files: &[CloudFile],
    selected_files: &[usize],
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> FileOperationResult {
    if selected_files.is_empty() {
        return FileOperationResult::Error("请选择要取消云同步的文件".to_string());
    }

    let filenames: Vec<String> = selected_files
        .iter()
        .filter_map(|&index| files.get(index).map(|f| f.name.clone()))
        .collect();

    let (forgotten_count, failed_files) = batch_forget_files(&filenames, steam_manager);

    if !failed_files.is_empty() {
        return FileOperationResult::Error(format!(
            "部分文件取消云同步失败: {}",
            failed_files.join(", ")
        ));
    }

    if forgotten_count > 0 {
        FileOperationResult::SuccessWithRefresh(format!("已取消云同步 {} 个文件", forgotten_count))
    } else {
        FileOperationResult::Error("没有文件被取消云同步".to_string())
    }
}

// 删除选中的文件
pub fn delete_selected_files_coordinated(
    files: &[CloudFile],
    selected_files: &[usize],
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> FileOperationResult {
    if selected_files.is_empty() {
        return FileOperationResult::Error("请选择要删除的文件".to_string());
    }

    let filenames: Vec<String> = selected_files
        .iter()
        .filter_map(|&index| files.get(index).map(|f| f.name.clone()))
        .collect();

    let (deleted_count, failed_files) = batch_delete_files(&filenames, steam_manager);

    if !failed_files.is_empty() {
        return FileOperationResult::Error(format!(
            "部分文件删除失败: {}",
            failed_files.join(", ")
        ));
    }

    if deleted_count > 0 {
        FileOperationResult::SuccessWithRefresh(format!("已删除 {} 个文件", deleted_count))
    } else {
        FileOperationResult::Error("没有文件被删除".to_string())
    }
}

// 上传文件
pub fn upload_files_with_dialog(
    _steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> Result<Option<UploadQueue>> {
    use rfd::FileDialog;

    // 多选文件
    if let Some(paths) = FileDialog::new()
        .add_filter("所有文件", &["*"])
        .pick_files()
    {
        let mut queue = UploadQueue::new();

        for path in paths {
            if let Err(e) = queue.add_file(path.clone()) {
                tracing::warn!("跳过文件 {}: {}", path.display(), e);
            }
        }

        if queue.total_files() > 0 {
            Ok(Some(queue))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

// 上传文件夹
pub fn upload_folder_with_dialog(
    _steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> Result<Option<UploadQueue>> {
    use rfd::FileDialog;

    // 选择文件夹
    if let Some(folder) = FileDialog::new().pick_folder() {
        let mut queue = UploadQueue::new();
        queue.add_folder(&folder)?;

        if queue.total_files() > 0 {
            Ok(Some(queue))
        } else {
            Err(anyhow!("文件夹中没有可上传的文件"))
        }
    } else {
        Ok(None)
    }
}

use std::path::Path;
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

// 常量定义
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

// 上传任务
#[derive(Debug, Clone)]
pub struct UploadTask {
    pub local_path: PathBuf,
    pub cloud_path: String,
    pub size: u64,
    pub status: TaskStatus,
    pub retry_count: usize,
    pub error: Option<String>,
}

// 任务状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Pending,         // 等待中
    Retrying(usize), // 重试中 (第N次)
    Success,         // 成功
    Failed,          // 失败
}

// 上传队列
pub struct UploadQueue {
    pub tasks: Vec<UploadTask>,
    pub virtual_root: Option<String>,
}

impl UploadQueue {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            virtual_root: None,
        }
    }

    // 添加单个文件
    pub fn add_file(&mut self, local_path: PathBuf) -> Result<()> {
        let size = std::fs::metadata(&local_path)?.len();

        // 验证文件大小
        if size > MAX_FILE_SIZE {
            return Err(anyhow!(
                "文件 {} 超过 100MB 限制 (实际: {})",
                local_path.display(),
                format_size(size)
            ));
        }

        let filename = local_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("无法获取文件名"))?;

        let cloud_path = if let Some(ref root) = self.virtual_root {
            format!("{}/{}", root, filename)
        } else {
            filename.to_string()
        };

        self.tasks.push(UploadTask {
            local_path,
            cloud_path,
            size,
            status: TaskStatus::Pending,
            retry_count: 0,
            error: None,
        });

        Ok(())
    }

    // 添加文件夹（递归）
    pub fn add_folder(&mut self, folder_path: &Path) -> Result<()> {
        for entry in WalkDir::new(folder_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let size = entry.metadata()?.len();

                // 跳过超大文件，但记录
                if size > MAX_FILE_SIZE {
                    tracing::warn!(
                        "跳过超大文件: {} ({})",
                        file_path.display(),
                        format_size(size)
                    );
                    continue;
                }

                // 计算相对路径
                let relative_path = file_path
                    .strip_prefix(folder_path)
                    .map_err(|e| anyhow!("无法计算相对路径: {}", e))?;

                // 统一使用 / 作为分隔符（跨平台）
                let cloud_path = if let Some(ref root) = self.virtual_root {
                    format!(
                        "{}/{}",
                        root,
                        relative_path.to_str().unwrap().replace("\\", "/")
                    )
                } else {
                    relative_path.to_str().unwrap().replace("\\", "/")
                };

                self.tasks.push(UploadTask {
                    local_path: file_path.to_path_buf(),
                    cloud_path,
                    size,
                    status: TaskStatus::Pending,
                    retry_count: 0,
                    error: None,
                });
            }
        }

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.tasks.iter().map(|t| t.size).sum()
    }

    pub fn total_files(&self) -> usize {
        self.tasks.len()
    }
}

// 格式化文件大小
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// 上传重试配置
pub struct UploadRetryConfig {
    pub max_retries: usize,
    pub retry_delay: Duration,
    pub backoff_multiplier: f32,
}

impl Default for UploadRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            backoff_multiplier: 1.5,
        }
    }
}

// 进度回调类型
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send>;

// 上传执行器
pub struct UploadExecutor {
    steam_manager: Arc<Mutex<crate::steam_api::SteamCloudManager>>,
    retry_config: UploadRetryConfig,
    progress_callback: Option<ProgressCallback>,
}

impl UploadExecutor {
    pub fn new(steam_manager: Arc<Mutex<crate::steam_api::SteamCloudManager>>) -> Self {
        Self {
            steam_manager,
            retry_config: UploadRetryConfig::default(),
            progress_callback: None,
        }
    }

    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, usize, &str) + Send + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    // 执行上传
    pub fn execute(&self, queue: &mut UploadQueue) -> Result<UploadResult> {
        let start_time = SystemTime::now();
        let mut success_count = 0;
        let mut failed_files = Vec::new();
        let total_size: u64 = queue.tasks.iter().map(|t| t.size).sum();
        let total_files = queue.tasks.len();

        for (index, task) in queue.tasks.iter_mut().enumerate() {
            // 发送进度更新
            if let Some(ref callback) = self.progress_callback {
                callback(index + 1, total_files, &task.cloud_path);
            }

            match self.upload_task_with_retry(task) {
                Ok(_) => {
                    task.status = TaskStatus::Success;
                    success_count += 1;
                }
                Err(e) => {
                    task.status = TaskStatus::Failed;
                    task.error = Some(e.to_string());
                    failed_files.push((task.cloud_path.clone(), e.to_string()));
                }
            }
        }

        let elapsed = start_time.elapsed().unwrap_or(Duration::from_secs(0));

        // 上传完成后，触发云同步
        if success_count > 0 {
            tracing::info!("上传完成，开始同步云文件...");
            if let Err(e) = self.sync_cloud_files() {
                tracing::warn!("云同步失败: {}", e);
            } else {
                tracing::info!("云同步已触发，Steam 将在后台同步文件");
            }
        }

        Ok(UploadResult {
            success_count,
            failed_count: failed_files.len(),
            total_size,
            elapsed_secs: elapsed.as_secs(),
            failed_files,
        })
    }

    // 上传单个任务
    fn upload_task_with_retry(&self, task: &mut UploadTask) -> Result<()> {
        let mut attempt = 0;
        let mut delay = self.retry_config.retry_delay;

        loop {
            attempt += 1;
            task.retry_count = attempt;

            // 读取文件数据
            let data = std::fs::read(&task.local_path)?;

            // 尝试上传
            match self.upload_to_steam(&task.cloud_path, &data) {
                Ok(_) => {
                    tracing::info!("文件上传成功: {} (尝试 {})", task.cloud_path, attempt);
                    return Ok(());
                }
                Err(e) => {
                    if attempt >= self.retry_config.max_retries {
                        tracing::error!(
                            "文件上传失败: {} (已重试 {} 次): {}",
                            task.cloud_path,
                            self.retry_config.max_retries,
                            e
                        );
                        return Err(e);
                    }

                    tracing::warn!(
                        "文件上传失败: {} (尝试 {}/{}): {}，{}秒后重试...",
                        task.cloud_path,
                        attempt,
                        self.retry_config.max_retries,
                        e,
                        delay.as_secs()
                    );

                    task.status = TaskStatus::Retrying(attempt);
                    std::thread::sleep(delay);
                    delay = Duration::from_secs_f32(
                        delay.as_secs_f32() * self.retry_config.backoff_multiplier,
                    );
                }
            }
        }
    }

    // 上传到 Steam
    fn upload_to_steam(&self, cloud_path: &str, data: &[u8]) -> Result<()> {
        let manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;

        manager.write_file(cloud_path, data)?;
        Ok(())
    }

    // 同步云文件
    fn sync_cloud_files(&self) -> Result<()> {
        let manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;

        manager.sync_cloud_files()?;
        Ok(())
    }
}

// 上传结果
#[derive(Debug, Clone)]
pub struct UploadResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub total_size: u64,
    pub elapsed_secs: u64,
    pub failed_files: Vec<(String, String)>,
}
