use crate::vdf_parser::UserInfo;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const STEAM_ID64_BASE: u64 = 76561197960265728;

// loginusers.vdf 中的一个账号
#[derive(Debug, Clone, PartialEq)]
pub struct LoginUser {
    pub user_id: String,
    pub account_name: Option<String>,
    pub persona_name: Option<String>,
    pub most_recent: bool,
    pub remember_password: bool,
    pub timestamp: i64,
}

// 解析 loginusers.vdf。键名大小写不固定（新版 Steam 写 "timestamp"，旧版写 "Timestamp"）
fn parse_loginusers(content: &str) -> Vec<LoginUser> {
    let mut users: Vec<LoginUser> = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split('"').collect();
        // 单独一行的 "<id64>" 开启一个账号块
        if parts.len() == 3
            && parts[0].trim().is_empty()
            && parts[2].trim().is_empty()
            && let Ok(id64) = parts[1].parse::<u64>()
        {
            if id64 > STEAM_ID64_BASE {
                users.push(LoginUser {
                    user_id: (id64 - STEAM_ID64_BASE).to_string(),
                    account_name: None,
                    persona_name: None,
                    most_recent: false,
                    remember_password: false,
                    timestamp: 0,
                });
            }
            continue;
        }

        // "key"  "value"
        if parts.len() >= 5
            && let Some(user) = users.last_mut()
        {
            let value = parts[3];
            match parts[1].to_ascii_lowercase().as_str() {
                "accountname" => user.account_name = Some(value.to_string()),
                "personaname" => user.persona_name = Some(value.to_string()),
                "mostrecent" => user.most_recent = value == "1",
                "rememberpassword" => user.remember_password = value == "1",
                "timestamp" => user.timestamp = value.parse().unwrap_or(0),
                _ => {}
            }
        }
    }

    users
}

pub fn read_loginusers(steam_path: &Path) -> Vec<LoginUser> {
    let p = steam_path.join("config").join("loginusers.vdf");
    match fs::read_to_string(&p) {
        Ok(s) => parse_loginusers(&s),
        Err(e) => {
            tracing::debug!("无法读取 {}: {}", p.display(), e);
            Vec::new()
        }
    }
}

// 当前登录账号：优先 MostRecent，其次最近登录时间
fn pick_current_user(users: &[LoginUser]) -> Option<&LoginUser> {
    users
        .iter()
        .find(|u| u.most_recent)
        .or_else(|| users.iter().max_by_key(|u| u.timestamp))
}

// 从 loginusers.vdf 查找当前登录用户的 ID 和昵称
pub fn find_user_id_from_loginusers(steam_path: &Path) -> Option<(String, Option<String>)> {
    let users = read_loginusers(steam_path);
    pick_current_user(&users).map(|u| (u.user_id.clone(), u.persona_name.clone()))
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

    let login_users: HashMap<String, String> = read_loginusers(steam_path)
        .into_iter()
        .filter_map(|u| u.persona_name.map(|name| (u.user_id, name)))
        .collect();

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

#[cfg(test)]
mod tests {
    use super::*;

    // 新版 Steam：小写 timestamp，无 MostRecent
    const LOGINUSERS_LOWERCASE: &str = r#""users"
{
	"76561198672877872"
	{
		"AccountName"		"alpha"
		"PersonaName"		"Alpha"
		"RememberPassword"		"1"
		"timestamp"		"1795021772"
	}
	"76561198384877484"
	{
		"AccountName"		"beta"
		"PersonaName"		"Beta"
		"RememberPassword"		"0"
		"timestamp"		"1700000000"
	}
}
"#;

    const LOGINUSERS_MOST_RECENT: &str = r#""users"
{
	"76561198672877872"
	{
		"AccountName"		"alpha"
		"PersonaName"		"Alpha"
		"MostRecent"		"0"
		"Timestamp"		"1795021772"
	}
	"76561198384877484"
	{
		"AccountName"		"beta"
		"PersonaName"		"Beta"
		"MostRecent"		"1"
		"Timestamp"		"1700000000"
	}
}
"#;

    #[test]
    fn picks_latest_timestamp_when_lowercase_and_not_last_block() {
        let users = parse_loginusers(LOGINUSERS_LOWERCASE);
        assert_eq!(users.len(), 2);
        let current = pick_current_user(&users).unwrap();
        assert_eq!(
            current.user_id,
            (76561198672877872 - STEAM_ID64_BASE).to_string()
        );
        assert_eq!(current.persona_name.as_deref(), Some("Alpha"));
        assert_eq!(current.account_name.as_deref(), Some("alpha"));
        assert!(current.remember_password);
        assert!(!users[1].remember_password);
    }

    #[test]
    fn most_recent_wins_over_timestamp() {
        let users = parse_loginusers(LOGINUSERS_MOST_RECENT);
        let current = pick_current_user(&users).unwrap();
        assert_eq!(current.account_name.as_deref(), Some("beta"));
        assert_eq!(current.persona_name.as_deref(), Some("Beta"));
    }
}
