use crate::path_resolver::ScannedLocalFile;
use crate::steam_api::CloudFile;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};

#[derive(Default)]
pub struct AsyncHandlers {
    pub loader_rx: Option<Receiver<Result<Vec<CloudFile>, String>>>,
    pub connect_rx: Option<Receiver<Result<u32, String>>>,
    pub scan_games_rx: Option<Receiver<Result<crate::game_scanner::ScanResult, String>>>,
    pub restart_rx: Option<Receiver<crate::steam_process::RestartStatus>>,
    pub upload_rx: Option<Receiver<Result<String, String>>>,
    pub upload_progress_rx: Option<Receiver<(usize, usize, String)>>,
    pub update_download_rx: Option<Receiver<Result<PathBuf, String>>>,
    pub backup_rx: Option<Receiver<crate::backup::BackupResult>>,
    pub backup_progress_rx: Option<Receiver<crate::backup::BackupProgress>>,
    pub backup_cancel: Option<Arc<AtomicBool>>,
    pub download_rx: Option<Receiver<crate::downloader::DownloadResult>>,
    pub download_progress_rx: Option<Receiver<crate::downloader::DownloadProgress>>,
    pub download_cancel: Option<Arc<AtomicBool>>,
    pub local_scan_rx: Option<Receiver<Vec<ScannedLocalFile>>>,
}

impl AsyncHandlers {
    pub fn cancel_backup(&self) {
        if let Some(ref flag) = self.backup_cancel {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn cancel_download(&self) {
        if let Some(ref flag) = self.download_cancel {
            flag.store(true, Ordering::Relaxed);
        }
    }
}

// 取结果型任务：线程没送出结果就退出时，返回 aborted 作为错误，让调用方复位状态
fn poll_result<T>(
    slot: &mut Option<Receiver<Result<T, String>>>,
    aborted: &str,
) -> Option<Result<T, String>> {
    let rx = slot.as_ref()?;
    match rx.try_recv() {
        Ok(result) => {
            *slot = None;
            Some(result)
        }
        Err(TryRecvError::Disconnected) => {
            *slot = None;
            Some(Err(aborted.to_string()))
        }
        Err(TryRecvError::Empty) => None,
    }
}

// 取状态或进度：发送端结束即任务结束
fn poll_value<T>(slot: &mut Option<Receiver<T>>) -> Option<T> {
    let rx = slot.as_ref()?;
    match rx.try_recv() {
        Ok(value) => Some(value),
        Err(TryRecvError::Disconnected) => {
            *slot = None;
            None
        }
        Err(TryRecvError::Empty) => None,
    }
}

impl AsyncHandlers {
    pub fn poll_connect(&mut self, aborted: &str) -> Option<Result<u32, String>> {
        poll_result(&mut self.connect_rx, aborted)
    }

    pub fn poll_loader(&mut self, aborted: &str) -> Option<Result<Vec<CloudFile>, String>> {
        poll_result(&mut self.loader_rx, aborted)
    }

    pub fn poll_scan_games(
        &mut self,
        aborted: &str,
    ) -> Option<Result<crate::game_scanner::ScanResult, String>> {
        poll_result(&mut self.scan_games_rx, aborted)
    }

    pub fn poll_restart(&mut self) -> Option<crate::steam_process::RestartStatus> {
        poll_value(&mut self.restart_rx)
    }

    pub fn poll_upload_progress(&mut self) -> Option<(usize, usize, String)> {
        poll_value(&mut self.upload_progress_rx)
    }

    pub fn poll_upload_result(&mut self, aborted: &str) -> Option<Result<String, String>> {
        poll_result(&mut self.upload_rx, aborted)
    }

    pub fn poll_update_download(&mut self, aborted: &str) -> Option<Result<PathBuf, String>> {
        poll_result(&mut self.update_download_rx, aborted)
    }

    pub fn poll_local_scan(&mut self) -> Option<Vec<ScannedLocalFile>> {
        let scanned = poll_value(&mut self.local_scan_rx);
        if scanned.is_some() {
            self.local_scan_rx = None;
        }
        scanned
    }
}

#[cfg(test)]
mod tests {
    use super::AsyncHandlers;

    #[test]
    fn connect_task_that_dies_without_a_result_reports_an_error() {
        let (tx, rx) = std::sync::mpsc::channel::<Result<u32, String>>();
        drop(tx);
        let mut handlers = AsyncHandlers {
            connect_rx: Some(rx),
            ..Default::default()
        };
        assert!(matches!(handlers.poll_connect("aborted"), Some(Err(e)) if e == "aborted"));
        assert!(handlers.connect_rx.is_none());
    }
}
