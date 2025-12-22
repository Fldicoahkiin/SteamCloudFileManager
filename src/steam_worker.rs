//Steam API 子进程 Worker
//在独立子进程中运行 Steam API，断开时杀死子进程，
//使 Steam 客户端立即识别游戏已退出。

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

// IPC 请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkerRequest {
    Connect { app_id: u32 },
    Disconnect,
    GetFiles,
    ReadFile { filename: String },
    WriteFile { filename: String, data: Vec<u8> },
    DeleteFile { filename: String },
    ForgetFile { filename: String },
    IsCloudEnabledForAccount,
    IsCloudEnabledForApp,
    SetCloudEnabledForApp { enabled: bool },
    SyncCloudFiles,
    Ping,
    Exit,
}

// IPC 响应
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkerResponse {
    Ok,
    Error { message: String },
    Connected { app_id: u32 },
    Files { files: Vec<WorkerCloudFile> },
    Quota { total: u64, available: u64 },
    FileData { data: Vec<u8> },
    Bool { value: bool },
    Pong,
}

// CloudFile 用于 IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCloudFile {
    pub name: String,
    pub size: u64,
    pub timestamp: i64,
    pub is_persisted: bool,
    pub exists: bool,
    pub root: u32,
    pub root_description: String,
}

// Worker 子进程管理器
pub struct SteamWorkerManager {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    response_rx: Option<Receiver<WorkerResponse>>,
    app_id: u32,
}

impl Default for SteamWorkerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SteamWorkerManager {
    pub fn new() -> Self {
        Self {
            child: None,
            stdin: None,
            response_rx: None,
            app_id: 0,
        }
    }

    // 启动 Worker 子进程
    fn spawn_worker(&mut self) -> Result<()> {
        if self.child.is_some() {
            return Ok(());
        }

        let exe_path = std::env::current_exe()?;
        tracing::info!("启动 Steam Worker 子进程: {:?}", exe_path);

        let mut child = Command::new(&exe_path)
            .arg("--steam-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| anyhow!("无法启动 Worker 子进程: {}", e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("无法获取 stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("无法获取 stdout"))?;

        // 创建响应接收线程
        let (tx, rx) = channel();
        thread::spawn(move || {
            Self::response_reader_thread(stdout, tx);
        });

        self.child = Some(child);
        self.stdin = Some(stdin);
        self.response_rx = Some(rx);

        // 等待 Worker 就绪
        self.send_request(&WorkerRequest::Ping)?;
        match self.receive_response_timeout(std::time::Duration::from_secs(5))? {
            WorkerResponse::Pong => {
                tracing::info!("Steam Worker 子进程已就绪");
                Ok(())
            }
            other => Err(anyhow!("Worker 启动失败，意外响应: {:?}", other)),
        }
    }

    // 响应读取线程
    fn response_reader_thread(stdout: ChildStdout, tx: Sender<WorkerResponse>) {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(json_line) => {
                    if json_line.trim().is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<WorkerResponse>(&json_line) {
                        Ok(response) => {
                            if tx.send(response).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("解析 Worker 响应失败: {} - {}", e, json_line);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("读取 Worker 输出失败: {}", e);
                    break;
                }
            }
        }
        tracing::debug!("Worker 响应读取线程退出");
    }

    // 发送请求到 Worker
    fn send_request(&mut self, request: &WorkerRequest) -> Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Worker 未启动"))?;
        let json = serde_json::to_string(request)?;
        writeln!(stdin, "{}", json)?;
        stdin.flush()?;
        Ok(())
    }

    // 接收响应
    fn receive_response_timeout(&self, timeout: std::time::Duration) -> Result<WorkerResponse> {
        let rx = self
            .response_rx
            .as_ref()
            .ok_or_else(|| anyhow!("Worker 未启动"))?;
        rx.recv_timeout(timeout)
            .map_err(|e| anyhow!("接收 Worker 响应超时: {}", e))
    }

    // 接收响应（默认超时 30 秒）
    fn receive_response(&self) -> Result<WorkerResponse> {
        self.receive_response_timeout(std::time::Duration::from_secs(30))
    }

    // 连接到 Steam（启动 Worker 子进程）
    pub fn connect(&mut self, app_id: u32) -> Result<()> {
        // 如果已连接到不同的 app_id，先断开
        if self.is_connected() && self.app_id != app_id {
            self.disconnect();
        }

        // 启动 Worker
        self.spawn_worker()?;

        // 发送连接请求
        self.send_request(&WorkerRequest::Connect { app_id })?;
        match self.receive_response()? {
            WorkerResponse::Connected {
                app_id: connected_id,
            } => {
                self.app_id = connected_id;
                tracing::info!("Worker 已连接到 Steam (App ID: {})", connected_id);
                Ok(())
            }
            WorkerResponse::Error { message } => Err(anyhow!("{}", message)),
            other => Err(anyhow!("连接失败，意外响应: {:?}", other)),
        }
    }

    // 异步连接
    pub fn connect_async(
        manager: std::sync::Arc<std::sync::Mutex<Self>>,
        app_id: u32,
    ) -> Receiver<Result<u32, String>> {
        let (tx, rx) = channel();

        thread::spawn(move || {
            let result = {
                let mut mgr = match manager.lock() {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = tx.send(Err(format!("锁错误: {}", e)));
                        return;
                    }
                };
                mgr.connect(app_id)
            };
            let _ = tx.send(result.map(|_| app_id).map_err(|e| e.to_string()));
        });

        rx
    }

    // 断开连接（杀死 Worker 子进程）
    pub fn disconnect(&mut self) {
        if let Some(mut child) = self.child.take() {
            tracing::info!(
                "断开 Steam 连接，终止 Worker 子进程 (App ID: {})",
                self.app_id
            );

            // 尝试优雅退出
            if let Some(mut stdin) = self.stdin.take() {
                let _ = writeln!(stdin, r#"{{"type":"Exit"}}"#);
                let _ = stdin.flush();
            }

            // 等待一小段时间让子进程优雅退出
            thread::sleep(std::time::Duration::from_millis(100));

            // 强制杀死子进程
            match child.try_wait() {
                Ok(Some(_)) => {
                    tracing::info!("Worker 子进程已优雅退出");
                }
                _ => {
                    tracing::info!("强制终止 Worker 子进程");
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }

            tracing::info!("Steam Worker 子进程已终止，Steam 应该识别游戏已退出");
        }

        self.stdin = None;
        self.response_rx = None;
        self.app_id = 0;
    }

    pub fn is_connected(&self) -> bool {
        self.child.is_some() && self.app_id > 0
    }

    // 通用请求-响应处理
    fn request<T, F>(&mut self, req: &WorkerRequest, extract: F) -> Result<T>
    where
        F: FnOnce(WorkerResponse) -> Result<T>,
    {
        self.send_request(req)?;
        let resp = self.receive_response()?;
        if let WorkerResponse::Error { message } = &resp {
            return Err(anyhow!("{}", message));
        }
        extract(resp)
    }

    pub fn get_files(&mut self) -> Result<Vec<WorkerCloudFile>> {
        self.request(&WorkerRequest::GetFiles, |r| match r {
            WorkerResponse::Files { files } => Ok(files),
            other => Err(anyhow!("意外响应: {:?}", other)),
        })
    }

    // 计算当前已用空间
    pub fn calculate_used_space(&mut self) -> Result<u64> {
        let files = self.get_files()?;
        let total_size: u64 = files.iter().map(|f| f.size).sum();
        Ok(total_size)
    }

    pub fn read_file(&mut self, filename: &str) -> Result<Vec<u8>> {
        self.request(
            &WorkerRequest::ReadFile {
                filename: filename.to_string(),
            },
            |r| match r {
                WorkerResponse::FileData { data } => Ok(data),
                other => Err(anyhow!("意外响应: {:?}", other)),
            },
        )
    }

    pub fn write_file(&mut self, filename: &str, data: &[u8]) -> Result<bool> {
        self.request(
            &WorkerRequest::WriteFile {
                filename: filename.to_string(),
                data: data.to_vec(),
            },
            |r| match r {
                WorkerResponse::Ok => Ok(true),
                other => Err(anyhow!("意外响应: {:?}", other)),
            },
        )
    }

    pub fn delete_file(&mut self, filename: &str) -> Result<bool> {
        self.request(
            &WorkerRequest::DeleteFile {
                filename: filename.to_string(),
            },
            |r| match r {
                WorkerResponse::Bool { value } => Ok(value),
                other => Err(anyhow!("意外响应: {:?}", other)),
            },
        )
    }

    pub fn forget_file(&mut self, filename: &str) -> Result<bool> {
        self.request(
            &WorkerRequest::ForgetFile {
                filename: filename.to_string(),
            },
            |r| match r {
                WorkerResponse::Bool { value } => Ok(value),
                other => Err(anyhow!("意外响应: {:?}", other)),
            },
        )
    }

    pub fn is_cloud_enabled_for_account(&mut self) -> Result<bool> {
        self.request(&WorkerRequest::IsCloudEnabledForAccount, |r| match r {
            WorkerResponse::Bool { value } => Ok(value),
            other => Err(anyhow!("意外响应: {:?}", other)),
        })
    }

    pub fn is_cloud_enabled_for_app(&mut self) -> Result<bool> {
        self.request(&WorkerRequest::IsCloudEnabledForApp, |r| match r {
            WorkerResponse::Bool { value } => Ok(value),
            other => Err(anyhow!("意外响应: {:?}", other)),
        })
    }

    pub fn set_cloud_enabled_for_app(&mut self, enabled: bool) -> Result<()> {
        self.request(
            &WorkerRequest::SetCloudEnabledForApp { enabled },
            |r| match r {
                WorkerResponse::Ok => Ok(()),
                other => Err(anyhow!("意外响应: {:?}", other)),
            },
        )
    }

    pub fn sync_cloud_files(&mut self) -> Result<()> {
        self.request(&WorkerRequest::SyncCloudFiles, |r| match r {
            WorkerResponse::Ok => Ok(()),
            other => Err(anyhow!("意外响应: {:?}", other)),
        })
    }

    pub fn run_callbacks(&self) {
        // Worker 子进程自动处理
    }
}

impl Drop for SteamWorkerManager {
    fn drop(&mut self) {
        self.disconnect();
    }
}

// Worker 子进程主循环
pub fn run_worker() {
    use std::io::stdin;

    let mut steam_manager = crate::steam_api::SteamCloudManager::new();
    let reader = BufReader::new(stdin());

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: WorkerRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let response = WorkerResponse::Error {
                    message: format!("解析请求失败: {}", e),
                };
                println!("{}", serde_json::to_string(&response).unwrap());
                continue;
            }
        };

        let response = handle_worker_request(&mut steam_manager, request);
        println!("{}", serde_json::to_string(&response).unwrap());

        // 如果是 Exit 请求，退出循环
        if matches!(response, WorkerResponse::Ok) {
            // 检查是否是 Exit 响应（通过之前的请求）
        }
    }
}

fn handle_worker_request(
    manager: &mut crate::steam_api::SteamCloudManager,
    request: WorkerRequest,
) -> WorkerResponse {
    match request {
        WorkerRequest::Ping => WorkerResponse::Pong,

        WorkerRequest::Exit => {
            manager.disconnect();
            std::process::exit(0);
        }

        WorkerRequest::Connect { app_id } => match manager.connect(app_id) {
            Ok(_) => WorkerResponse::Connected { app_id },
            Err(e) => WorkerResponse::Error {
                message: e.to_string(),
            },
        },

        WorkerRequest::Disconnect => {
            manager.disconnect();
            WorkerResponse::Ok
        }

        WorkerRequest::GetFiles => match manager.get_files_from_api() {
            Ok(files) => {
                let worker_files: Vec<WorkerCloudFile> = files
                    .into_iter()
                    .map(|f| WorkerCloudFile {
                        name: f.name,
                        size: f.size,
                        timestamp: f.timestamp.timestamp(),
                        is_persisted: f.is_persisted,
                        exists: f.exists,
                        root: f.root,
                        root_description: f.root_description,
                    })
                    .collect();
                WorkerResponse::Files {
                    files: worker_files,
                }
            }
            Err(e) => WorkerResponse::Error {
                message: e.to_string(),
            },
        },

        WorkerRequest::ReadFile { filename } => match manager.read_file(&filename) {
            Ok(data) => WorkerResponse::FileData { data },
            Err(e) => WorkerResponse::Error {
                message: e.to_string(),
            },
        },

        WorkerRequest::WriteFile { filename, data } => match manager.write_file(&filename, &data) {
            Ok(_) => WorkerResponse::Ok,
            Err(e) => WorkerResponse::Error {
                message: e.to_string(),
            },
        },

        WorkerRequest::DeleteFile { filename } => match manager.delete_file(&filename) {
            Ok(result) => WorkerResponse::Bool { value: result },
            Err(e) => WorkerResponse::Error {
                message: e.to_string(),
            },
        },

        WorkerRequest::ForgetFile { filename } => match manager.forget_file(&filename) {
            Ok(result) => WorkerResponse::Bool { value: result },
            Err(e) => WorkerResponse::Error {
                message: e.to_string(),
            },
        },

        WorkerRequest::IsCloudEnabledForAccount => match manager.is_cloud_enabled_for_account() {
            Ok(enabled) => WorkerResponse::Bool { value: enabled },
            Err(e) => WorkerResponse::Error {
                message: e.to_string(),
            },
        },

        WorkerRequest::IsCloudEnabledForApp => match manager.is_cloud_enabled_for_app() {
            Ok(enabled) => WorkerResponse::Bool { value: enabled },
            Err(e) => WorkerResponse::Error {
                message: e.to_string(),
            },
        },

        WorkerRequest::SetCloudEnabledForApp { enabled } => {
            match manager.set_cloud_enabled_for_app(enabled) {
                Ok(_) => WorkerResponse::Ok,
                Err(e) => WorkerResponse::Error {
                    message: e.to_string(),
                },
            }
        }

        WorkerRequest::SyncCloudFiles => match manager.sync_cloud_files() {
            Ok(_) => WorkerResponse::Ok,
            Err(e) => WorkerResponse::Error {
                message: e.to_string(),
            },
        },
    }
}
