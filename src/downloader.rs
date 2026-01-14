use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

use crate::steam_api::CloudFile;

// 下载进度
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub total_files: usize,
    pub completed_files: usize,
    pub current_file: String,
    pub failed_files: Vec<(String, String)>,
}

impl DownloadProgress {
    pub fn new(total_files: usize) -> Self {
        Self {
            total_files,
            completed_files: 0,
            current_file: String::new(),
            failed_files: Vec::new(),
        }
    }

    pub fn percent(&self) -> f32 {
        if self.total_files == 0 {
            return 100.0;
        }
        (self.completed_files as f32 / self.total_files as f32) * 100.0
    }
}

// 下载结果
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub success: bool,
    pub target_dir: PathBuf,
    pub total_files: usize,
    pub success_count: usize,
    pub failed_files: Vec<(String, String)>,
}

// 下载任务
pub struct DownloadTask {
    pub file: CloudFile,
    pub target_path: PathBuf,
    pub local_save_paths: Vec<(String, PathBuf)>,
}

// 批量下载执行器
pub struct BatchDownloader {
    tasks: Vec<DownloadTask>,
    cancel_flag: Arc<AtomicBool>,
    progress_tx: Option<Sender<DownloadProgress>>,
    steam_manager: Option<Arc<std::sync::Mutex<crate::steam_worker::SteamWorkerManager>>>,
}

impl BatchDownloader {
    pub fn new(tasks: Vec<DownloadTask>) -> Self {
        Self {
            tasks,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            progress_tx: None,
            steam_manager: None,
        }
    }

    pub fn with_cancel_flag(mut self, flag: Arc<AtomicBool>) -> Self {
        self.cancel_flag = flag;
        self
    }

    pub fn with_progress_sender(mut self, tx: Sender<DownloadProgress>) -> Self {
        self.progress_tx = Some(tx);
        self
    }

    pub fn with_steam_manager(
        mut self,
        manager: Arc<std::sync::Mutex<crate::steam_worker::SteamWorkerManager>>,
    ) -> Self {
        self.steam_manager = Some(manager);
        self
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Relaxed)
    }

    fn send_progress(&self, progress: &DownloadProgress) {
        if let Some(ref tx) = self.progress_tx {
            let _ = tx.send(progress.clone());
        }
    }

    // 执行批量下载
    pub fn execute(self) -> DownloadResult {
        let total_files = self.tasks.len();
        let mut progress = DownloadProgress::new(total_files);
        let mut success_count = 0;
        let target_dir = self
            .tasks
            .first()
            .and_then(|t| t.target_path.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        for task in &self.tasks {
            // 检查取消
            if self.is_cancelled() {
                tracing::info!("下载已取消");
                break;
            }

            progress.current_file = task.file.name.clone();
            self.send_progress(&progress);

            // 执行下载
            let result = download_file_full(
                &task.file,
                &task.target_path,
                &task.local_save_paths,
                self.steam_manager.as_ref(),
            );

            match result {
                Ok(_) => {
                    success_count += 1;
                    tracing::debug!("下载成功: {}", task.file.name);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    tracing::warn!("下载失败: {} - {}", task.file.name, err_msg);
                    progress
                        .failed_files
                        .push((task.file.name.clone(), err_msg));
                }
            }

            progress.completed_files += 1;
            self.send_progress(&progress);
        }

        let cancelled = self.is_cancelled();
        DownloadResult {
            success: progress.failed_files.is_empty() && !cancelled,
            target_dir,
            total_files,
            success_count,
            failed_files: progress.failed_files,
        }
    }
}

// 完整下载单个文件（CDP -> Steam API -> 本地复制）
pub fn download_file_full(
    file: &CloudFile,
    target_path: &Path,
    local_save_paths: &[(String, PathBuf)],
    steam_manager: Option<&Arc<std::sync::Mutex<crate::steam_worker::SteamWorkerManager>>>,
) -> Result<()> {
    // 创建父目录
    if let Some(parent) = target_path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    // 使用 CDP URL 下载
    if file.root_description.starts_with("CDP:") {
        let content = &file.root_description[4..];
        let url = content.split('|').next().unwrap_or("");

        if !url.is_empty() {
            tracing::debug!("CDP 下载: {} -> {}", file.name, target_path.display());

            match ureq::get(url).call() {
                Ok(resp) => {
                    let mut data = Vec::new();
                    std::io::Read::read_to_end(&mut resp.into_body().into_reader(), &mut data)
                        .map_err(|e| anyhow!("读取响应失败: {}", e))?;
                    std::fs::write(target_path, &data)?;
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("CDP 下载失败，尝试其他方式: {}", e);
                }
            }
        }
    }

    // Steam API 下载
    if file.is_persisted
        && let Some(manager) = steam_manager
    {
        tracing::debug!("使用 Steam API 下载: {}", file.name);
        if let Ok(mut mgr) = manager.lock()
            && let Ok(data) = mgr.read_file(&file.name)
        {
            std::fs::write(target_path, &data)?;
            return Ok(());
        }
    }

    // 从本地复制
    if file.exists {
        let file_root_desc = if file.root_description.starts_with("CDP:") {
            file.root_description
                .split('|')
                .nth(1)
                .unwrap_or(&file.root_description)
        } else {
            &file.root_description
        };

        for (desc, base_path) in local_save_paths {
            if desc == file_root_desc || file_root_desc.contains(desc) {
                let local_file_path = base_path.join(&file.name);
                if local_file_path.exists() {
                    tracing::debug!("从本地复制: {} -> {:?}", file.name, local_file_path);
                    let data = std::fs::read(&local_file_path)?;
                    std::fs::write(target_path, &data)?;
                    return Ok(());
                }
            }
        }

        // 直接匹配文件名
        for (_desc, base_path) in local_save_paths {
            let local_file_path = base_path.join(&file.name);
            if local_file_path.exists() {
                tracing::debug!("从本地复制: {} -> {:?}", file.name, local_file_path);
                let data = std::fs::read(&local_file_path)?;
                std::fs::write(target_path, &data)?;
                return Ok(());
            }
        }
    }

    Err(anyhow!("文件既不在云端也无法从本地找到"))
}

// 仅 CDP 下载（用于备份）
pub fn download_single_file(file: &CloudFile, target_path: &Path) -> Result<()> {
    download_file_full(file, target_path, &[], None)
}
