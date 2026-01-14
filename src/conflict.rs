use chrono::{DateTime, Local, TimeZone};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, mpsc};

// 文件同步状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncStatus {
    #[default]
    Unknown, // 未知状态
    Synced,     // 完全同步
    LocalNewer, // 本地较新
    CloudNewer, // 云端较新
    Conflict,   // 冲突（需要用户决定）
    LocalOnly,  // 仅本地存在
    CloudOnly,  // 仅云端存在
}

// Hash 检测状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HashStatus {
    #[default]
    Pending, // 等待检测
    Checking, // 正在检测
    Match,    // Hash 匹配
    Mismatch, // Hash 不匹配
    Error,    // 检测出错
}

// 各项差异标记
#[derive(Debug, Clone, Default)]
pub struct DiffFlags {
    pub exists_diff: bool,    // 存在状态不同
    pub persisted_diff: bool, // 同步状态不同
    pub size_diff: bool,      // 大小不同
    pub time_diff: bool,      // 时间不同
    pub hash_diff: bool,      // Hash 不同
}

// 本地文件信息
#[derive(Debug, Clone)]
pub struct LocalFileInfo {
    pub size: u64,
    pub modified: DateTime<Local>,
    pub exists: bool,
    pub hash: Option<String>, // 本地文件 SHA1 hash
}

// 云端文件信息
#[derive(Debug, Clone)]
pub struct CloudFileInfo {
    pub size: u64,
    pub timestamp: DateTime<Local>,
    pub is_persisted: bool,
    pub hash: Option<String>,         // 云端文件 SHA1 hash（需要下载计算）
    pub download_url: Option<String>, // CDP 下载链接
}

// 文件对比结果
#[derive(Debug, Clone)]
pub struct FileComparison {
    pub filename: String,
    pub status: SyncStatus,
    pub local: Option<LocalFileInfo>,
    pub cloud: Option<CloudFileInfo>,
    pub time_diff_secs: i64,
    pub size_diff_bytes: i64,
    pub diff_flags: DiffFlags,
    pub hash_status: HashStatus,
    pub local_path: Option<PathBuf>, // 本地文件完整路径（用于 hash 计算）
}

impl FileComparison {
    pub fn new(
        filename: String,
        local: Option<LocalFileInfo>,
        cloud: Option<CloudFileInfo>,
        download_url: Option<String>,
        local_path: Option<PathBuf>,
    ) -> Self {
        let (status, time_diff_secs, size_diff_bytes, diff_flags) =
            Self::calculate_status(&local, &cloud);

        // 更新云端信息的下载链接
        let cloud = cloud.map(|mut c| {
            c.download_url = download_url;
            c
        });

        Self {
            filename,
            status,
            local,
            cloud,
            time_diff_secs,
            size_diff_bytes,
            diff_flags,
            hash_status: HashStatus::Pending,
            local_path,
        }
    }

    fn calculate_status(
        local: &Option<LocalFileInfo>,
        cloud: &Option<CloudFileInfo>,
    ) -> (SyncStatus, i64, i64, DiffFlags) {
        let mut flags = DiffFlags::default();

        match (local, cloud) {
            (Some(l), Some(c)) => {
                // 同时检查本地和云端状态
                match (l.exists, c.is_persisted) {
                    (false, false) => {
                        // 本地和云端都不存在 → Steam 缓存残留，标记为未知
                        flags.exists_diff = true;
                        flags.persisted_diff = true;
                        return (SyncStatus::Unknown, 0, 0, flags);
                    }
                    (false, true) => {
                        // 本地不存在，但云端存在 → 仅云端
                        flags.exists_diff = true;
                        return (SyncStatus::CloudOnly, 0, 0, flags);
                    }
                    (true, false) => {
                        // 本地存在，但云端未同步 → 仅本地
                        flags.persisted_diff = true;
                        return (SyncStatus::LocalOnly, 0, 0, flags);
                    }
                    (true, true) => {
                        // 两边都存在，继续比较
                    }
                }

                // 计算大小差异（按精度最低的来，即 CDP 精度：1% 或最小 1KB）
                let size_diff = l.size as i64 - c.size as i64;
                let size_tolerance = (c.size as f64 * 0.01).max(1024.0) as i64;
                flags.size_diff = size_diff.abs() > size_tolerance;

                // 计算时间差异（精确到秒）
                let time_diff = l.modified.timestamp() - c.timestamp.timestamp();
                flags.time_diff = time_diff != 0;

                // 判断状态
                let status = if time_diff > 2 {
                    SyncStatus::LocalNewer
                } else if time_diff < -2 {
                    SyncStatus::CloudNewer
                } else if flags.size_diff {
                    // 时间相同但大小不同 → 需要 hash 确认
                    SyncStatus::Unknown
                } else {
                    SyncStatus::Synced
                };

                (status, time_diff, size_diff, flags)
            }
            (Some(l), None) if l.exists => {
                flags.exists_diff = true;
                (SyncStatus::LocalOnly, 0, 0, flags)
            }
            (None, Some(c)) if c.is_persisted => {
                flags.exists_diff = true;
                (SyncStatus::CloudOnly, 0, 0, flags)
            }
            _ => (SyncStatus::Unknown, 0, 0, flags),
        }
    }

    pub fn status_display(&self) -> String {
        match self.status {
            SyncStatus::Synced => format!("{} 已同步", crate::icons::CHECK),
            SyncStatus::LocalNewer => format!("{} 本地较新", crate::icons::ARROW_UP),
            SyncStatus::CloudNewer => format!("{} 云端较新", crate::icons::ARROW_DOWN),
            SyncStatus::Conflict => format!("{} 冲突", crate::icons::WARNING),
            SyncStatus::LocalOnly => format!("{} 仅本地", crate::icons::FILE),
            SyncStatus::CloudOnly => format!("{} 仅云端", crate::icons::CLOUD),
            SyncStatus::Unknown => format!("{} 检测中", crate::icons::QUESTION),
        }
    }

    pub fn hash_status_display(&self) -> String {
        match self.hash_status {
            HashStatus::Pending => format!("{} 等待", crate::icons::HOURGLASS),
            HashStatus::Checking => format!("{} 检测中", crate::icons::SPINNER),
            HashStatus::Match => format!("{} 一致", crate::icons::CHECK),
            HashStatus::Mismatch => format!("{} 不一致", crate::icons::ERROR),
            HashStatus::Error => format!("{} 错误", crate::icons::WARNING),
        }
    }
}

// 批量检测文件同步状态
pub fn detect_all(
    cloud_files: &[crate::steam_api::CloudFile],
    local_save_paths: &[(String, PathBuf)],
) -> Vec<FileComparison> {
    tracing::info!("开始文件对比检测，共 {} 个云端文件", cloud_files.len());

    let comparisons: Vec<FileComparison> = cloud_files
        .iter()
        .map(|cf| {
            let local_base_path = find_local_path_for_file(cf, local_save_paths);

            // 提取下载 URL
            let (download_url, _) =
                crate::path_resolver::parse_cdp_root_description(&cf.root_description);
            let download_url = download_url.map(|s| s.to_string());

            // 计算完整本地路径
            let full_local_path = local_base_path.as_ref().map(|p| p.join(&cf.name));

            let cloud_info = CloudFileInfo {
                size: cf.size,
                timestamp: cf.timestamp,
                is_persisted: cf.is_persisted,
                hash: None,
                download_url: None,
            };
            let local_info = local_base_path.and_then(|p| get_local_file_info(&p.join(&cf.name)));
            FileComparison::new(
                cf.name.clone(),
                local_info,
                Some(cloud_info),
                download_url,
                full_local_path,
            )
        })
        .collect();

    // 统计并输出日志
    let mut synced = 0;
    let mut local_newer = 0;
    let mut cloud_newer = 0;
    let mut conflicts = 0;
    let mut local_only = 0;
    let mut cloud_only = 0;
    let mut unknown = 0;

    for c in &comparisons {
        match c.status {
            SyncStatus::Synced => synced += 1,
            SyncStatus::LocalNewer => local_newer += 1,
            SyncStatus::CloudNewer => cloud_newer += 1,
            SyncStatus::Conflict => conflicts += 1,
            SyncStatus::LocalOnly => local_only += 1,
            SyncStatus::CloudOnly => cloud_only += 1,
            SyncStatus::Unknown => unknown += 1,
        }
    }

    tracing::info!(
        "对比完成: 已同步={}, 本地较新={}, 云端较新={}, 冲突={}, 仅本地={}, 仅云端={}, 待检测={}",
        synced,
        local_newer,
        cloud_newer,
        conflicts,
        local_only,
        cloud_only,
        unknown
    );

    comparisons
}

pub fn find_local_path_for_file(
    cloud_file: &crate::steam_api::CloudFile,
    local_save_paths: &[(String, PathBuf)],
) -> Option<PathBuf> {
    // 优先尝试通过 root ID 匹配 (最可靠)
    // local_save_paths 的 desc 格式是 "TypeName (ID)"，所以我们查找以 "(ID)" 结尾的项
    let target_id_suffix = format!("({})", cloud_file.root);
    for (desc, base_path) in local_save_paths {
        if desc.ends_with(&target_id_suffix) {
            return Some(base_path.clone());
        }
    }

    // 尝试通过 root_description 字符串匹配 (兼容旧逻辑或处理特殊情况)
    // 获取去除 CDP 前缀后的描述符
    let (_, file_root_desc) =
        crate::path_resolver::parse_cdp_root_description(&cloud_file.root_description);

    for (desc, base_path) in local_save_paths {
        // 如果描述符完全相等，或者文件描述符包含了本地路径描述符（反之亦然）
        if desc == file_root_desc || file_root_desc.contains(desc) {
            return Some(base_path.clone());
        }
    }

    // 默认回退
    local_save_paths.first().map(|(_, p)| p.clone())
}

fn get_local_file_info(path: &PathBuf) -> Option<LocalFileInfo> {
    if !path.exists() {
        return Some(LocalFileInfo {
            size: 0,
            modified: Local::now(),
            exists: false,
            hash: None,
        });
    }

    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| {
            let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
            Local.timestamp_opt(duration.as_secs() as i64, 0).single()
        })
        .unwrap_or_else(Local::now);

    Some(LocalFileInfo {
        size: metadata.len(),
        modified,
        exists: true,
        hash: None, // Hash 会在异步检测时填充
    })
}

// Hash 检测任务
#[derive(Debug, Clone)]
pub struct HashCheckTask {
    pub filename: String,
    pub local_path: Option<PathBuf>,
    pub download_url: Option<String>,
}

// Hash 检测结果
#[derive(Debug, Clone)]
pub struct HashCheckResult {
    pub filename: String,
    pub local_hash: Option<String>,
    pub cloud_hash: Option<String>,
    pub error: Option<String>,
}

impl HashCheckResult {
    // 处理 hash 检测结果，返回 (hash_status, sync_status_changed)
    pub fn process(&self) -> (HashStatus, Option<SyncStatus>) {
        if self.error.is_some() {
            return (HashStatus::Error, None);
        }

        match (&self.local_hash, &self.cloud_hash) {
            (Some(lh), Some(ch)) if lh == ch => (HashStatus::Match, Some(SyncStatus::Synced)),
            (Some(_), Some(_)) => (HashStatus::Mismatch, Some(SyncStatus::Conflict)),
            _ => (HashStatus::Error, None),
        }
    }
}

// 异步 Hash 检测器
pub struct AsyncHashChecker {
    app_id: Arc<AtomicU32>,
    cancelled: Arc<AtomicBool>,
    tasks: Arc<Mutex<Vec<HashCheckTask>>>,
    results: Arc<Mutex<HashMap<String, HashCheckResult>>>,
    progress: Arc<Mutex<(usize, usize)>>, // (completed, total)
    result_rx: Option<mpsc::Receiver<HashCheckResult>>,
}

impl AsyncHashChecker {
    pub fn new() -> Self {
        Self {
            app_id: Arc::new(AtomicU32::new(0)),
            cancelled: Arc::new(AtomicBool::new(false)),
            tasks: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            progress: Arc::new(Mutex::new((0, 0))),
            result_rx: None,
        }
    }

    // 开始检测新的 appid
    pub fn start(&mut self, app_id: u32, comparisons: &[FileComparison]) {
        // 取消之前的检测
        self.cancel();

        self.app_id.store(app_id, Ordering::SeqCst);
        self.cancelled.store(false, Ordering::SeqCst);

        // 收集需要检测的任务
        let tasks: Vec<HashCheckTask> = comparisons
            .iter()
            .filter(|c| {
                // 只检测需要 hash 确认的文件（本地和云端都存在）
                c.local.as_ref().map(|l| l.exists).unwrap_or(false)
                    && c.cloud.as_ref().map(|c| c.is_persisted).unwrap_or(false)
                    && c.local_path.is_some()
            })
            .map(|c| HashCheckTask {
                filename: c.filename.clone(),
                local_path: c.local_path.clone(),
                download_url: c.cloud.as_ref().and_then(|c| c.download_url.clone()),
            })
            .collect();

        let total = tasks.len();
        *self.progress.lock().unwrap() = (0, total);
        *self.tasks.lock().unwrap() = tasks.clone();
        self.results.lock().unwrap().clear();

        if total == 0 {
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.result_rx = Some(rx);

        let cancelled = Arc::clone(&self.cancelled);
        let tasks = Arc::clone(&self.tasks);
        let progress = Arc::clone(&self.progress);

        // 启动后台线程
        std::thread::spawn(move || {
            let tasks = tasks.lock().unwrap().clone();

            for (idx, task) in tasks.iter().enumerate() {
                if cancelled.load(Ordering::SeqCst) {
                    tracing::debug!("Hash 检测已取消");
                    break;
                }

                let result = check_file_hash(task);

                if tx.send(result).is_err() {
                    break;
                }

                *progress.lock().unwrap() = (idx + 1, tasks.len());
            }
        });
    }

    // 取消当前检测
    pub fn cancel(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.result_rx = None;
    }

    // 轮询结果
    pub fn poll(&mut self) -> Vec<HashCheckResult> {
        let mut results = Vec::new();

        if let Some(rx) = &self.result_rx {
            while let Ok(result) = rx.try_recv() {
                results.push(result.clone());
                self.results
                    .lock()
                    .unwrap()
                    .insert(result.filename.clone(), result);
            }
        }

        results
    }

    // 检查是否完成所有检测
    pub fn is_completed(&self) -> bool {
        let (completed, total) = *self.progress.lock().unwrap();
        total > 0 && completed >= total
    }

    // 检查是否正在检测中
    pub fn is_running(&self) -> bool {
        let (completed, total) = *self.progress.lock().unwrap();
        total > 0 && completed < total && !self.cancelled.load(Ordering::SeqCst)
    }

    // 获取当前检测的 app_id
    pub fn get_app_id(&self) -> u32 {
        self.app_id.load(Ordering::SeqCst)
    }
}

impl Default for AsyncHashChecker {
    fn default() -> Self {
        Self::new()
    }
}

// 检测单个文件的 hash
fn check_file_hash(task: &HashCheckTask) -> HashCheckResult {
    let mut result = HashCheckResult {
        filename: task.filename.clone(),
        local_hash: None,
        cloud_hash: None,
        error: None,
    };

    // 计算本地文件 hash
    if let Some(ref path) = task.local_path {
        match calculate_file_hash(path) {
            Ok(hash) => result.local_hash = Some(hash),
            Err(e) => {
                result.error = Some(format!("本地文件 hash 计算失败: {}", e));
                return result;
            }
        }
    }

    // 下载云端文件并计算 hash
    if let Some(ref url) = task.download_url {
        match download_and_hash(url) {
            Ok(hash) => result.cloud_hash = Some(hash),
            Err(e) => {
                result.error = Some(format!("云端文件 hash 计算失败: {}", e));
            }
        }
    }

    result
}

// 计算文件 hash
fn calculate_file_hash(path: &PathBuf) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

// 下载文件并计算 hash
fn download_and_hash(url: &str) -> Result<String, String> {
    let resp = ureq::get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .call()
        .map_err(|e| format!("下载失败: {}", e))?;

    let mut data = Vec::new();
    resp.into_body()
        .into_reader()
        .read_to_end(&mut data)
        .map_err(|e| format!("读取失败: {}", e))?;

    let mut hasher = Sha1::new();
    hasher.update(&data);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
