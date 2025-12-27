use anyhow::{anyhow, Result};
use std::process::Command;
use std::sync::mpsc::Sender;

// Steam 重启状态
#[derive(Debug, Clone)]
pub enum RestartStatus {
    Closing,
    Starting,
    Success,
    Error(String),
}

// 检测 Steam 是否正在运行
pub fn is_steam_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        // 使用 WinAPI
        unsafe {
            use std::ffi::CStr;
            use winapi::um::handleapi::CloseHandle;
            use winapi::um::tlhelp32::{
                CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
                TH32CS_SNAPPROCESS,
            };

            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
                return false;
            }

            let mut entry: PROCESSENTRY32 = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

            if Process32First(snapshot, &mut entry) != 0 {
                loop {
                    // 转换 szExeFile 为字符串
                    let exe_name = CStr::from_ptr(entry.szExeFile.as_ptr()).to_string_lossy();
                    if exe_name.eq_ignore_ascii_case("steam.exe") {
                        CloseHandle(snapshot);
                        return true;
                    }

                    if Process32Next(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);
            false
        }
    }

    #[cfg(target_os = "macos")]
    {
        let result = Command::new("pgrep")
            .arg("-f")
            .arg("/Applications/Steam.app")
            .output();

        if let Ok(output) = result {
            if output.status.success() && !output.stdout.is_empty() {
                return true;
            }
        }

        Command::new("pgrep")
            .arg("-x")
            .arg("steam_osx")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("pgrep")
            .arg("-x")
            .arg("steam")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

// 等待 Steam 进程关闭
fn wait_for_steam_shutdown(max_wait_secs: u64) -> bool {
    let start = std::time::Instant::now();
    let max_duration = std::time::Duration::from_secs(max_wait_secs);

    while start.elapsed() < max_duration {
        if !is_steam_running() {
            tracing::info!("确认 Steam 进程已关闭");
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    tracing::warn!("Steam 进程关闭超时，可能未完全关闭");
    false
}

// 等待 Steam 进程启动
fn wait_for_steam_startup(max_wait_secs: u64) -> bool {
    let start = std::time::Instant::now();
    let max_duration = std::time::Duration::from_secs(max_wait_secs);

    tracing::info!("等待 Steam 启动...");

    while start.elapsed() < max_duration {
        if is_steam_running() {
            let elapsed = start.elapsed().as_secs();
            tracing::info!("确认 Steam 进程已启动（耗时 {} 秒）", elapsed);
            return true;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    tracing::warn!("Steam 进程启动超时");
    false
}

// 带状态回调的 Steam 重启函数
pub fn restart_steam_with_status<F>(tx: Sender<RestartStatus>, on_update: F)
where
    F: Fn() + Send + 'static,
{
    // 发送关闭状态
    let _ = tx.send(RestartStatus::Closing);
    on_update();

    // 执行关闭操作
    let close_result = close_steam();

    if let Err(e) = close_result {
        let error_msg = e.to_string();
        if error_msg.contains("MANUAL_OPERATION_REQUIRED") {
            let _ = tx.send(RestartStatus::Error("自动关闭失败".to_string()));
        } else {
            let _ = tx.send(RestartStatus::Error(error_msg));
        }
        on_update();
        return;
    }

    // 发送启动状态
    let _ = tx.send(RestartStatus::Starting);
    on_update();

    // 执行启动操作
    let start_result = start_steam();

    if let Err(e) = start_result {
        let _ = tx.send(RestartStatus::Error(e.to_string()));
        on_update();
        return;
    }

    // 发送成功状态
    let _ = tx.send(RestartStatus::Success);
    on_update();
}

// 关闭 Steam
fn close_steam() -> Result<()> {
    if !is_steam_running() {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        close_steam_macos()
    }

    #[cfg(target_os = "windows")]
    {
        close_steam_windows()
    }

    #[cfg(target_os = "linux")]
    {
        close_steam_linux()
    }
}

// 启动 Steam
fn start_steam() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        start_steam_macos()
    }

    #[cfg(target_os = "windows")]
    {
        start_steam_windows()
    }

    #[cfg(target_os = "linux")]
    {
        start_steam_linux()
    }
}

#[cfg(target_os = "macos")]
fn close_steam_macos() -> Result<()> {
    tracing::info!("正在关闭 macOS 上的 Steam 进程...");

    // 使用 AppleScript 优雅退出
    let quit_script = r#"
        tell application "Steam"
            quit
        end tell
    "#;

    Command::new("osascript")
        .arg("-e")
        .arg(quit_script)
        .status()
        .ok();

    // 等待一下
    std::thread::sleep(std::time::Duration::from_secs(2));

    // 如果还在运行，尝试强制杀死
    if is_steam_running() {
        tracing::warn!("Steam 未响应退出命令，尝试强制结束进程...");
        Command::new("pkill")
            .arg("-9")
            .arg("-f")
            .arg("/Applications/Steam.app")
            .status()
            .ok();
        Command::new("pkill")
            .arg("-9")
            .arg("-x")
            .arg("steam_osx")
            .status()
            .ok();
    }

    // 等待进程关闭
    if !wait_for_steam_shutdown(5) {
        tracing::warn!("自动关闭 Steam 失败，需要手动操作");
        return Err(anyhow!("MANUAL_OPERATION_REQUIRED"));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn start_steam_macos() -> Result<()> {
    tracing::info!("正在启动 Steam，添加参数: -cef-enable-debugging");
    Command::new("open")
        .arg("-a")
        .arg("Steam")
        .arg("--args")
        .arg("-cef-enable-debugging")
        .spawn()
        .map_err(|e| anyhow!("无法启动 Steam: {}", e))?;

    // 等待进程启动
    if !wait_for_steam_startup(30) {
        return Err(anyhow!(
            "Steam 启动超时，请检查 Steam 是否正常运行\n\n如果 Steam 未启动，请手动执行：\nopen -a Steam --args -cef-enable-debugging"
        ));
    }

    tracing::info!("Steam 已成功启动");
    Ok(())
}

#[cfg(target_os = "windows")]
fn close_steam_windows() -> Result<()> {
    tracing::info!("正在关闭 Windows 上的 Steam 进程...");
    use std::os::windows::process::CommandExt;
    Command::new("taskkill")
        .args(["/F", "/IM", "steam.exe"])
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
        .status()
        .ok();

    if !wait_for_steam_shutdown(5) {
        return Err(anyhow!("MANUAL_OPERATION_REQUIRED"));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn start_steam_windows() -> Result<()> {
    let steam_dir = crate::vdf_parser::VdfParser::find_steam_path()?;
    let steam_exe = steam_dir.join("steam.exe");

    if !steam_exe.exists() {
        return Err(anyhow!("找不到 steam.exe"));
    }

    tracing::info!(path = ?steam_exe, "正在启动 Steam，添加参数: -cef-enable-debugging");
    Command::new(&steam_exe)
        .arg("-cef-enable-debugging")
        .spawn()
        .map_err(|e| anyhow!("无法启动 Steam: {}", e))?;

    // 等待进程启动
    if !wait_for_steam_startup(30) {
        return Err(anyhow!(
            "Steam 启动超时，请检查 Steam 是否正常运行\n\n如果 Steam 未启动，请手动启动并添加启动参数：-cef-enable-debugging"
        ));
    }

    tracing::info!("Steam 已成功启动");
    Ok(())
}

#[cfg(target_os = "linux")]
fn close_steam_linux() -> Result<()> {
    tracing::info!("正在关闭 Linux 上的 Steam 进程...");
    Command::new("pkill").arg("-x").arg("steam").status().ok();

    if !wait_for_steam_shutdown(5) {
        return Err(anyhow!("MANUAL_OPERATION_REQUIRED"));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn start_steam_linux() -> Result<()> {
    tracing::info!("正在启动 Steam，添加参数: -cef-enable-debugging");

    // 使用 sh -c 在后台启动 Steam，避免阻塞和闪退
    let result = Command::new("sh")
        .arg("-c")
        .arg("nohup steam -cef-enable-debugging >/dev/null 2>&1 &")
        .spawn();

    match result {
        Ok(_) => {
            // 给 Steam 一点时间启动
            std::thread::sleep(std::time::Duration::from_secs(2));

            // 等待进程启动
            if !wait_for_steam_startup(30) {
                return Err(anyhow!(
                    "Steam 启动超时，请检查 Steam 是否正常运行\n\n如果 Steam 未启动，请手动在终端执行：\nsteam -cef-enable-debugging"
                ));
            }

            tracing::info!("Steam 已成功启动");
            Ok(())
        }
        Err(e) => {
            tracing::warn!("使用 nohup 启动失败，尝试直接启动: {}", e);

            // 降级方案：直接启动但分离进程
            Command::new("steam")
                .arg("-cef-enable-debugging")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| anyhow!("无法启动 Steam: {}", e))?;

            std::thread::sleep(std::time::Duration::from_secs(2));

            if !wait_for_steam_startup(30) {
                return Err(anyhow!(
                    "Steam 启动超时，请检查 Steam 是否正常运行\n\n如果 Steam 未启动，请手动在终端执行：\nsteam -cef-enable-debugging"
                ));
            }

            tracing::info!("Steam 已成功启动");
            Ok(())
        }
    }
}
