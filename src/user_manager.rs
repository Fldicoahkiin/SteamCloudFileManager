use crate::vdf_parser::UserInfo;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// 从 loginusers.vdf 查找用户 ID
pub fn find_user_id_from_loginusers(steam_path: &Path) -> Option<(String, Option<String>)> {
    let p = steam_path.join("config").join("loginusers.vdf");
    let s = fs::read_to_string(&p).ok()?;
    let mut current_id64: Option<u64> = None;
    let mut most_recent_id64: Option<u64> = None;
    let mut most_recent_name: Option<String> = None;
    let mut most_recent_timestamp: i64 = 0;

    for line in s.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('"')
            && trimmed.ends_with('"')
            && let Ok(id64) = trimmed.trim_matches('"').parse::<u64>()
        {
            current_id64 = Some(id64);
        }

        if let Some(id64) = current_id64 {
            if trimmed.contains("\"PersonaName\"")
                && let Some(name) = trimmed.split('"').nth(3)
                && (trimmed.contains("\"MostRecent\"") || trimmed.contains("\"Timestamp\""))
                && let Some(ts_str) = trimmed.split('"').nth(3)
                && let Ok(ts) = ts_str.parse::<i64>()
                && ts > most_recent_timestamp
            {
                most_recent_timestamp = ts;
                most_recent_id64 = Some(id64);
                most_recent_name = Some(name.to_string());
            }

            if trimmed.contains("\"PersonaName\"")
                && let Some(name) = trimmed.split('"').nth(3)
            {
                most_recent_name = Some(name.to_string());
            }

            if trimmed.contains("\"Timestamp\"")
                && let Some(ts_str) = trimmed.split('"').nth(3)
                && let Ok(ts) = ts_str.parse::<i64>()
                && ts > most_recent_timestamp
            {
                most_recent_timestamp = ts;
                most_recent_id64 = Some(id64);
            }
        }
    }

    let id64 = most_recent_id64.or(current_id64)?;
    let base: u64 = 76561197960265728;
    if id64 > base {
        Some(((id64 - base).to_string(), most_recent_name))
    } else {
        None
    }
}

// 查找用户 ID
pub fn find_user_id(steam_path: &Path) -> Result<(String, Option<String>)> {
    if let Some((uid, name)) = find_user_id_from_loginusers(steam_path) {
        return Ok((uid, name));
    }
    let userdata_path = steam_path.join("userdata");
    if let Ok(entries) = fs::read_dir(&userdata_path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && name.parse::<u64>().is_ok()
            {
                return Ok((name.to_string(), None));
            }
        }
    }
    Err(anyhow!("未找到用户ID"))
}

// 获取所有用户信息
pub fn get_all_users_info(steam_path: &Path, current_user_id: &str) -> Result<Vec<UserInfo>> {
    let userdata_path = steam_path.join("userdata");
    let mut users = Vec::new();

    // 读取 loginusers.vdf 获取所有用户的昵称信息
    let mut login_users = HashMap::new();
    let loginusers_path = steam_path.join("config").join("loginusers.vdf");
    if let Ok(content) = fs::read_to_string(&loginusers_path) {
        let mut current_id64: Option<u64> = None;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('"')
                && trimmed.ends_with('"')
                && let Ok(id64) = trimmed.trim_matches('"').parse::<u64>()
            {
                current_id64 = Some(id64);
            }

            if let Some(id64) = current_id64
                && trimmed.contains("\"PersonaName\"")
                && let Some(name) = trimmed.split('"').nth(3)
            {
                let base: u64 = 76561197960265728;
                if id64 > base {
                    let user_id = (id64 - base).to_string();
                    login_users.insert(user_id, name.to_string());
                }
            }
        }
    }

    // 扫描 userdata 目录
    if let Ok(entries) = fs::read_dir(&userdata_path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && name.parse::<u64>().is_ok()
            {
                let user_id = name.to_string();
                let persona_name = login_users.get(&user_id).cloned();
                let is_current = user_id == current_user_id;

                users.push(UserInfo {
                    user_id,
                    persona_name,
                    is_current,
                });
            }
        }
    }

    // 按当前用户优先排序
    users.sort_by(|a, b| {
        if a.is_current {
            std::cmp::Ordering::Less
        } else if b.is_current {
            std::cmp::Ordering::Greater
        } else {
            a.user_id.cmp(&b.user_id)
        }
    });

    Ok(users)
}
