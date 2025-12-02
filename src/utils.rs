// 将字节数格式化为易读的大小字符串
pub fn format_size(bytes: u64) -> String {
    let bytes_f = bytes as f64;
    if bytes_f < 1024.0 {
        format!("{} B", bytes)
    } else if bytes_f < 1024.0 * 1024.0 {
        format!("{:.2} KB", bytes_f / 1024.0)
    } else if bytes_f < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} MB", bytes_f / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes_f / (1024.0 * 1024.0 * 1024.0))
    }
}

// 解析大小字符串为字节数（如 "1.5 MB" -> 1572864）
pub fn parse_size(s: &str) -> u64 {
    let s = s.replace(",", "").to_lowercase();
    let s = s.replace("\u{a0}", " ");
    let parts: Vec<&str> = s.split_whitespace().collect();

    if parts.is_empty() {
        return 0;
    }

    let num = parts[0].parse::<f64>().unwrap_or(0.0);
    if parts.len() > 1 {
        match parts[1] {
            "kb" | "k" => (num * 1024.0) as u64,
            "mb" | "m" => (num * 1024.0 * 1024.0) as u64,
            "gb" | "g" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
            "b" | "bytes" => num as u64,
            _ => num as u64,
        }
    } else {
        num as u64
    }
}

// 打开文件夹
pub fn open_folder(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}
