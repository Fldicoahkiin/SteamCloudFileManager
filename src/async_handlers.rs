use crate::steam_api::CloudFile;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;

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

impl AsyncHandlers {
    pub fn poll_connect(&mut self) -> Option<Result<u32, String>> {
        if let Some(rx) = &self.connect_rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.connect_rx = None;
                    Some(result)
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.connect_rx = None;
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
            }
        } else {
            None
        }
    }

    pub fn poll_loader(&mut self) -> Option<Result<Vec<CloudFile>, String>> {
        if let Some(rx) = &self.loader_rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.loader_rx = None;
                    Some(result)
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.loader_rx = None;
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
            }
        } else {
            None
        }
    }

    pub fn poll_scan_games(&mut self) -> Option<Result<crate::game_scanner::ScanResult, String>> {
        if let Some(rx) = &self.scan_games_rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.scan_games_rx = None;
                    Some(result)
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.scan_games_rx = None;
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
            }
        } else {
            None
        }
    }

    pub fn poll_restart(&mut self) -> Option<crate::steam_process::RestartStatus> {
        if let Some(rx) = &self.restart_rx {
            match rx.try_recv() {
                Ok(status) => Some(status),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.restart_rx = None;
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
            }
        } else {
            None
        }
    }

    pub fn poll_upload_progress(&mut self) -> Option<(usize, usize, String)> {
        if let Some(rx) = &self.upload_progress_rx {
            match rx.try_recv() {
                Ok(progress) => Some(progress),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.upload_progress_rx = None;
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
            }
        } else {
            None
        }
    }

    pub fn poll_upload_result(&mut self) -> Option<Result<String, String>> {
        if let Some(rx) = &self.upload_rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.upload_rx = None;
                    Some(result)
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.upload_rx = None;
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
            }
        } else {
            None
        }
    }

    pub fn poll_update_download(&mut self) -> Option<Result<PathBuf, String>> {
        if let Some(rx) = &self.update_download_rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.update_download_rx = None;
                    Some(result)
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.update_download_rx = None;
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
            }
        } else {
            None
        }
    }
}
