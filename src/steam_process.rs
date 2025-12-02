use anyhow::{anyhow, Result};
use std::process::Command;

// 检测 Steam 是否正在运行
pub fn is_steam_running() -> bool {
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

    #[cfg(target_os = "windows")]
    {
        Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq steam.exe", "/NH"])
            .output()
            .map(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                output.to_lowercase().contains("steam.exe")
                    && !output.contains("INFO:")
                    && !output.contains("没有运行")
            })
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

// 以调试模式重启 Steam
pub fn restart_steam_with_debugging() -> Result<()> {
    let steam_running = is_steam_running();

    if steam_running {
        tracing::info!("检测到 Steam 正在运行，开始重启...");
    } else {
        tracing::info!("Steam 未运行，直接以调试模式启动...");
    }

    #[cfg(target_os = "macos")]
    {
        restart_steam_macos()
    }

    #[cfg(target_os = "windows")]
    {
        restart_steam_windows()
    }

    #[cfg(target_os = "linux")]
    {
        restart_steam_linux()
    }
}

#[cfg(target_os = "macos")]
fn restart_steam_macos() -> Result<()> {
    // 检测 Steam 是否运行
    if is_steam_running() {
        tracing::info!("正在关闭 macOS 上的 Steam 进程...");

        // 关闭现有 Steam
        Command::new("pkill")
            .arg("-f")
            .arg("/Applications/Steam.app")
            .status()
            .ok();
        Command::new("pkill")
            .arg("-x")
            .arg("steam_osx")
            .status()
            .ok();

        // 等待进程关闭
        if !wait_for_steam_shutdown(10) {
            return Err(anyhow!(
                "Steam 进程关闭超时，请手动关闭 Steam 后重试\n\n手动操作：\n1. 右键 Dock 图标 -> 退出\n2. 打开终端，执行：open -a Steam --args -cef-enable-debugging"
            ));
        }
    } else {
        tracing::info!("Steam 未运行，直接启动...");
    }

    tracing::info!("正在启动 Steam，添加参数: -cef-enable-debugging");
    Command::new("open")
        .arg("-a")
        .arg("Steam")
        .arg("--args")
        .arg("-cef-enable-debugging")
        .spawn()
        .map_err(|e| {
            tracing::error!(error = %e, "启动 Steam 失败");
            anyhow!(
                "无法启动 Steam: {}\n\n手动操作：\n1. 打开终端（Terminal）\n2. 执行命令：open -a Steam --args -cef-enable-debugging",
                e
            )
        })?;

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
fn restart_steam_windows() -> Result<()> {
    // 检测 Steam 是否运行
    if is_steam_running() {
        tracing::info!("正在关闭 Windows 上的 Steam 进程...");

        // 关闭现有 Steam
        Command::new("taskkill")
            .args(["/F", "/IM", "steam.exe"])
            .status()
            .ok();

        // 等待进程关闭
        if !wait_for_steam_shutdown(10) {
            return Err(anyhow!(
                "Steam 进程关闭超时，请手动关闭 Steam 后重试\n\n手动操作：\n1. 右键 Steam 快捷方式 -> 属性\n2. 在目标栏末尾添加：-cef-enable-debugging\n3. 点击确定并启动 Steam"
            ));
        }
    } else {
        tracing::info!("Steam 未运行，直接启动...");
    }

    // 找到并启动 Steam
    tracing::info!("正在查找 Steam 安装路径...");
    let steam_dir = crate::vdf_parser::VdfParser::find_steam_path()?;
    let steam_exe = steam_dir.join("steam.exe");

    if !steam_exe.exists() {
        tracing::error!(path = ?steam_exe, "找不到 steam.exe");
        return Err(anyhow!(
            "找不到 steam.exe，路径: {:?}\n\n请确认 Steam 已正确安装",
            steam_exe
        ));
    }

    tracing::info!(path = ?steam_exe, "正在启动 Steam，添加参数: -cef-enable-debugging");

    // 使用 cmd /C start 来启动
    Command::new("cmd")
        .args(["/C", "start", ""])
        .arg(&steam_exe)
        .arg("-cef-enable-debugging")
        .spawn()
        .map_err(|e| {
            tracing::error!(error = %e, "启动 Steam 失败");
            anyhow!(
                "无法启动 Steam: {}\n\n手动操作：\n1. 右键 Steam 快捷方式 -> 属性\n2. 在目标栏末尾添加：-cef-enable-debugging\n3. 点击确定并启动 Steam",
                e
            )
        })?;

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
fn restart_steam_linux() -> Result<()> {
    // 检测 Steam 是否运行
    if is_steam_running() {
        tracing::info!("正在关闭 Linux 上的 Steam 进程...");

        Command::new("pkill").arg("-x").arg("steam").status().ok();

        // 等待进程关闭
        if !wait_for_steam_shutdown(10) {
            return Err(anyhow!(
                "Steam 进程关闭超时，请手动关闭 Steam 后重试\n\n手动操作：\n1. 在终端执行：pkill steam\n2. 然后执行：steam -cef-enable-debugging"
            ));
        }
    } else {
        tracing::info!("Steam 未运行，直接启动...");
    }

    tracing::info!("正在启动 Steam，添加参数: -cef-enable-debugging");

    // 尝试从 PATH 启动
    let spawn_result = Command::new("steam").arg("-cef-enable-debugging").spawn();

    if spawn_result.is_err() {
        tracing::warn!("PATH 中找不到 steam 命令，尝试其他路径...");

        // 尝试常见路径
        let common_paths = ["/usr/bin/steam", "/usr/games/steam", "/usr/local/bin/steam"];

        let mut found = false;
        for path in &common_paths {
            if std::path::Path::new(path).exists() {
                tracing::info!("在 {} 找到 steam", path);
                if Command::new(path)
                    .arg("-cef-enable-debugging")
                    .spawn()
                    .is_ok()
                {
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return Err(anyhow!(
                "无法启动 Steam，请手动在终端执行：\nsteam -cef-enable-debugging\n\n或者尝试：\n/usr/bin/steam -cef-enable-debugging"
            ));
        }
    }

    // 等待进程启动
    if !wait_for_steam_startup(30) {
        return Err(anyhow!(
            "Steam 启动超时，请检查 Steam 是否正常运行\n\n如果 Steam 未启动，请手动在终端执行：\nsteam -cef-enable-debugging"
        ));
    }

    tracing::info!("Steam 已成功启动");
    Ok(())
}
