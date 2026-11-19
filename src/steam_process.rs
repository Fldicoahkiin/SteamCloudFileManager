use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
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
                CreateToolhelp32Snapshot, PROCESSENTRY32, Process32First, Process32Next,
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

        if let Ok(output) = result
            && output.status.success()
            && !output.stdout.is_empty()
        {
            return true;
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
            tracing::debug!("确认 Steam 进程已关闭");
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

    tracing::debug!("等待 Steam 启动...");

    while start.elapsed() < max_duration {
        if is_steam_running() {
            let elapsed = start.elapsed().as_secs();
            tracing::debug!("确认 Steam 进程已启动（耗时 {} 秒）", elapsed);
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
    restart_steam_inner(tx, on_update, || Ok(()));
}

// 切换 Steam 账号：关闭 Steam，设置自动登录账号，再带调试参数启动。
// 必须在 Steam 退出后写入，否则 Steam 退出时会覆盖 loginusers.vdf / registry.vdf。
pub fn switch_account_with_status<F>(
    tx: Sender<RestartStatus>,
    steam_path: PathBuf,
    user_id: String,
    account_name: String,
    on_update: F,
) where
    F: Fn() + Send + 'static,
{
    // 关闭 Steam 之前先确认能写入，避免关掉 Steam 后才发现切换不了
    if let Err(e) = check_auto_login_writable(&steam_path, &account_name) {
        tracing::error!("无法切换 Steam 账号: {}", e);
        let _ = tx.send(RestartStatus::Error(e.to_string()));
        on_update();
        return;
    }
    restart_steam_inner(tx, on_update, move || {
        set_auto_login_user(&steam_path, &user_id, &account_name)
    });
}

fn restart_steam_inner<F, B>(tx: Sender<RestartStatus>, on_update: F, before_start: B)
where
    F: Fn() + Send + 'static,
    B: FnOnce() -> Result<()>,
{
    // 发送关闭状态
    let _ = tx.send(RestartStatus::Closing);
    on_update();

    // 执行关闭操作
    let close_result = close_steam();

    if let Err(e) = close_result {
        let error_msg = e.to_string();
        if error_msg.contains("MANUAL_OPERATION_REQUIRED") {
            let _ = tx.send(RestartStatus::Error("Auto close failed".to_string()));
        } else {
            let _ = tx.send(RestartStatus::Error(error_msg));
        }
        on_update();
        return;
    }

    // 准备失败也要把 Steam 启动回来，错误在启动后再上报
    let prepare_result = before_start();
    if let Err(e) = &prepare_result {
        tracing::error!("启动 Steam 前的准备失败: {}", e);
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

    if let Err(e) = prepare_result {
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
    tracing::debug!("正在关闭 macOS 上的 Steam 进程...");

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
    tracing::debug!("正在启动 Steam，添加参数: -cef-enable-debugging");
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
    tracing::debug!("正在关闭 Windows 上的 Steam 进程...");
    use std::os::windows::process::CommandExt;

    // 先让 Steam 自己退出，保存配置；强制结束只作为兜底。
    // 新版 Steam 退出前会等刚结束的游戏完成云同步，所以多等一会再强制结束
    if let Ok(steam_dir) = crate::vdf_parser::VdfParser::find_steam_path() {
        let steam_exe = steam_dir.join("steam.exe");
        if let Err(e) = Command::new(&steam_exe)
            .arg("-shutdown")
            .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
            .spawn()
        {
            tracing::warn!("调用 steam.exe -shutdown 失败: {}", e);
        } else if wait_for_steam_shutdown(60) {
            return Ok(());
        }
    }

    tracing::warn!("Steam 未正常退出，强制结束进程");
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

    tracing::debug!(path = ?steam_exe, "正在启动 Steam，添加参数: -cef-enable-debugging");
    // 与本进程分离启动，不继承标准输入输出，避免 Steam 的生命周期受本程序影响
    use std::os::windows::process::CommandExt;
    use std::process::Stdio;
    Command::new(&steam_exe)
        .arg("-cef-enable-debugging")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(
            winapi::um::winbase::DETACHED_PROCESS | winapi::um::winbase::CREATE_NEW_PROCESS_GROUP,
        )
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
    tracing::debug!("正在关闭 Linux 上的 Steam 进程...");
    Command::new("pkill").arg("-x").arg("steam").status().ok();

    if !wait_for_steam_shutdown(5) {
        return Err(anyhow!("MANUAL_OPERATION_REQUIRED"));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn start_steam_linux() -> Result<()> {
    tracing::debug!("正在启动 Steam，添加参数: -cef-enable-debugging");

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

// Steam 账号名只允许字母、数字和下划线；写入 VDF / 注册表前校验，避免引号等字符破坏文件
fn is_valid_account_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(target_os = "macos")]
fn registry_vdf_path(steam_path: &Path) -> Result<PathBuf> {
    Ok(steam_path.join("registry.vdf"))
}

#[cfg(target_os = "linux")]
fn registry_vdf_path(_steam_path: &Path) -> Result<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| anyhow!("无法获取 HOME 目录"))?;
    Ok(PathBuf::from(home).join(".steam").join("registry.vdf"))
}

fn check_auto_login_writable(steam_path: &Path, account_name: &str) -> Result<()> {
    if !is_valid_account_name(account_name) {
        return Err(anyhow!("账号名不合法: {}", account_name));
    }
    let loginusers = steam_path.join("config").join("loginusers.vdf");
    std::fs::read_to_string(&loginusers)
        .map_err(|e| anyhow!("无法读取 {}: {}", loginusers.display(), e))?;

    #[cfg(not(target_os = "windows"))]
    {
        let path = registry_vdf_path(steam_path)?;
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow!("无法读取 {}: {}", path.display(), e))?;
        if replace_vdf_value(&content, "AutoLoginUser", account_name).is_none() {
            return Err(anyhow!(
                "{} 中没有 AutoLoginUser，请在 Steam 中手动切换账号",
                path.display()
            ));
        }
    }
    Ok(())
}

// 先写自动登录账号，再写 loginusers.vdf：前者失败时 loginusers.vdf 保持原样
fn set_auto_login_user(steam_path: &Path, user_id: &str, account_name: &str) -> Result<()> {
    tracing::info!("设置 Steam 自动登录账号: user_id={}", user_id);

    #[cfg(target_os = "windows")]
    set_windows_auto_login_user(account_name)?;

    #[cfg(not(target_os = "windows"))]
    set_registry_vdf_auto_login_user(&registry_vdf_path(steam_path)?, account_name)?;

    let loginusers = steam_path.join("config").join("loginusers.vdf");
    let content = std::fs::read_to_string(&loginusers)
        .map_err(|e| anyhow!("无法读取 {}: {}", loginusers.display(), e))?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    std::fs::write(&loginusers, mark_login_user_current(&content, user_id, now))
        .map_err(|e| anyhow!("无法写入 {}: {}", loginusers.display(), e))
}

// 把目标账号标记为当前账号：已有的 MostRecent 改为目标 1、其余 0，目标的 timestamp 改为 now。
// 不插入新的 MostRecent：新版 Steam 不写这个键，插入后若被保留会长期压过 timestamp。
// Steam 登录后会用真实时间覆盖 timestamp。
fn mark_login_user_current(content: &str, user_id: &str, now: i64) -> String {
    const STEAM_ID64_BASE: u64 = 76561197960265728;
    let target_id64 = user_id
        .parse::<u64>()
        .map(|id| (id + STEAM_ID64_BASE).to_string())
        .unwrap_or_default();

    let mut out = String::with_capacity(content.len() + 32);
    let mut depth = 0usize;
    let mut block_id: Option<String> = None;
    let mut block_has_timestamp = false;

    for line in content.lines() {
        let trimmed = line.trim();
        let parts: Vec<&str> = trimmed.split('"').collect();
        let in_target = block_id.as_deref() == Some(target_id64.as_str());
        let indent = &line[..line.len() - line.trim_start().len()];

        if trimmed == "{" {
            depth += 1;
        } else if trimmed == "}" {
            if depth == 2 && in_target && !block_has_timestamp {
                out.push_str(&format!("\t\t\"timestamp\"\t\t\"{}\"\n", now));
            }
            if depth == 2 {
                block_id = None;
            }
            depth = depth.saturating_sub(1);
        } else if depth == 1 && parts.len() == 3 {
            block_id = Some(parts[1].to_string());
            block_has_timestamp = false;
        } else if depth == 2 && parts.len() >= 5 && parts[1].eq_ignore_ascii_case("MostRecent") {
            let value = if in_target { "1" } else { "0" };
            out.push_str(&format!("{}\"{}\"\t\t\"{}\"\n", indent, parts[1], value));
            continue;
        } else if depth == 2
            && in_target
            && parts.len() >= 5
            && parts[1].eq_ignore_ascii_case("timestamp")
        {
            block_has_timestamp = true;
            out.push_str(&format!("{}\"{}\"\t\t\"{}\"\n", indent, parts[1], now));
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    out
}

#[cfg(not(target_os = "windows"))]
fn set_registry_vdf_auto_login_user(path: &Path, account_name: &str) -> Result<()> {
    let content =
        std::fs::read_to_string(path).map_err(|e| anyhow!("无法读取 {}: {}", path.display(), e))?;
    let updated = replace_vdf_value(&content, "AutoLoginUser", account_name).ok_or_else(|| {
        anyhow!(
            "{} 中没有 AutoLoginUser，请在 Steam 中手动切换账号",
            path.display()
        )
    })?;
    std::fs::write(path, updated).map_err(|e| anyhow!("无法写入 {}: {}", path.display(), e))
}

// 替换第一个匹配键的值，保留原缩进
#[cfg(any(not(target_os = "windows"), test))]
fn replace_vdf_value(content: &str, key: &str, value: &str) -> Option<String> {
    let mut replaced = false;
    let mut out = String::with_capacity(content.len());
    for line in content.lines() {
        let parts: Vec<&str> = line.trim().split('"').collect();
        if !replaced && parts.len() >= 5 && parts[1].eq_ignore_ascii_case(key) {
            let indent = &line[..line.len() - line.trim_start().len()];
            out.push_str(&format!("{}\"{}\"\t\t\"{}\"\n", indent, parts[1], value));
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    replaced.then_some(out)
}

#[cfg(target_os = "windows")]
fn set_windows_auto_login_user(account_name: &str) -> Result<()> {
    use std::ptr::null_mut;
    use winapi::um::winnt::{KEY_SET_VALUE, REG_DWORD, REG_SZ};
    use winapi::um::winreg::{HKEY_CURRENT_USER, RegCloseKey, RegOpenKeyExA, RegSetValueExA};

    let account = format!("{}\0", account_name);
    let remember: u32 = 1;
    // SAFETY: 所有字符串以 \0 结尾；hkey 仅在打开成功后使用并关闭
    unsafe {
        let mut hkey: winapi::shared::minwindef::HKEY = null_mut();
        let result = RegOpenKeyExA(
            HKEY_CURRENT_USER,
            "Software\\Valve\\Steam\0".as_ptr() as *const i8,
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );
        if result != 0 {
            return Err(anyhow!(
                "无法打开注册表 HKCU\\Software\\Valve\\Steam (错误码 {})",
                result
            ));
        }

        let r1 = RegSetValueExA(
            hkey,
            "AutoLoginUser\0".as_ptr() as *const i8,
            0,
            REG_SZ,
            account.as_ptr(),
            account.len() as u32,
        );
        let r2 = RegSetValueExA(
            hkey,
            "RememberPassword\0".as_ptr() as *const i8,
            0,
            REG_DWORD,
            &remember as *const u32 as *const u8,
            std::mem::size_of::<u32>() as u32,
        );
        RegCloseKey(hkey);

        if r1 != 0 || r2 != 0 {
            return Err(anyhow!("写入注册表失败 (错误码 {} / {})", r1, r2));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOGINUSERS: &str = "\"users\"\n{\n\t\"76561198672877872\"\n\t{\n\t\t\"AccountName\"\t\t\"alpha\"\n\t\t\"MostRecent\"\t\t\"1\"\n\t}\n\t\"76561198384877484\"\n\t{\n\t\t\"AccountName\"\t\t\"beta\"\n\t\t\"timestamp\"\t\t\"1700000000\"\n\t}\n}\n";

    #[test]
    fn marks_target_current_without_inserting_most_recent() {
        let beta_id = (76561198384877484u64 - 76561197960265728).to_string();
        let out = mark_login_user_current(LOGINUSERS, &beta_id, 1800000000);
        let expected = "\"users\"\n{\n\t\"76561198672877872\"\n\t{\n\t\t\"AccountName\"\t\t\"alpha\"\n\t\t\"MostRecent\"\t\t\"0\"\n\t}\n\t\"76561198384877484\"\n\t{\n\t\t\"AccountName\"\t\t\"beta\"\n\t\t\"timestamp\"\t\t\"1800000000\"\n\t}\n}\n";
        assert_eq!(out, expected);
        // 切换后按 user_manager 的规则也应选中目标账号
        let alpha_id = (76561198672877872u64 - 76561197960265728).to_string();
        let out = mark_login_user_current(LOGINUSERS, &alpha_id, 1800000000);
        assert!(out.contains("\"MostRecent\"\t\t\"1\""));
    }

    #[test]
    fn replaces_auto_login_user_keeping_indent() {
        let content = "\"Steam\"\n{\n\t\t\t\t\t\"AutoLoginUser\"\t\t\"alpha\"\n}\n";
        let out = replace_vdf_value(content, "AutoLoginUser", "beta").unwrap();
        assert_eq!(
            out,
            "\"Steam\"\n{\n\t\t\t\t\t\"AutoLoginUser\"\t\t\"beta\"\n}\n"
        );
        assert!(replace_vdf_value("\"a\"\t\"b\"\n", "AutoLoginUser", "x").is_none());
    }

    #[test]
    fn rejects_account_names_that_would_break_vdf() {
        assert!(is_valid_account_name("user_01"));
        assert!(!is_valid_account_name("bad\"name"));
        assert!(!is_valid_account_name(""));
    }
}
