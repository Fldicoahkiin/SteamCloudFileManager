use crate::steam_api::CloudFile;
use crate::vdf_parser::CloudGameInfo;
use anyhow::{anyhow, Result};
use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::TcpStream;
use tungstenite::{stream::MaybeTlsStream, WebSocket};

#[derive(Debug, Deserialize)]
struct Target {
    #[serde(rename = "webSocketDebuggerUrl")]
    websocket_debugger_url: Option<String>,
    #[serde(rename = "type")]
    target_type: String,
}

pub struct CdpClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    message_id: u64,
}

#[derive(Serialize)]
struct CdpCommand {
    id: u64,
    method: String,
    params: Value,
}

#[derive(Deserialize)]
struct CdpResponse {
    id: u64,
    result: Option<Value>,
    error: Option<Value>,
}

impl CdpClient {
    // 检查 CDP 服务是否可用
    pub fn is_cdp_running() -> bool {
        ureq::get("http://127.0.0.1:8080/json").call().is_ok()
    }

    // 连接到 Steam CDP 端口
    pub fn connect() -> Result<Self> {
        // 获取目标列表
        let resp = ureq::get("http://127.0.0.1:8080/json")
            .call()
            .map_err(|e| anyhow!("无法连接到 Steam 调试端口: {}", e))?;

        let text = resp.into_string()?;
        let targets: Vec<Target> = serde_json::from_str(&text)?;

        let target = targets
            .iter()
            .find(|t| t.websocket_debugger_url.is_some() && t.target_type == "page")
            .or_else(|| targets.iter().find(|t| t.websocket_debugger_url.is_some()))
            .ok_or_else(|| anyhow!("未找到可用的调试目标"))?;

        let ws_url = target.websocket_debugger_url.as_ref().unwrap();
        let (socket, _) = tungstenite::connect(ws_url)?;

        Ok(Self {
            socket,
            message_id: 0,
        })
    }

    // 发送命令并等待响应
    fn send_command(&mut self, method: &str, params: Value) -> Result<Value> {
        self.message_id += 1;
        let id = self.message_id;

        let command = CdpCommand {
            id,
            method: method.to_string(),
            params,
        };

        let msg = serde_json::to_string(&command)?;
        self.socket.send(tungstenite::Message::Text(msg))?;

        loop {
            let msg = self.socket.read()?;
            if let tungstenite::Message::Text(text) = msg {
                if let Ok(resp) = serde_json::from_str::<CdpResponse>(&text) {
                    if resp.id == id {
                        if let Some(error) = resp.error {
                            return Err(anyhow!("CDP 错误: {:?}", error));
                        }
                        return Ok(resp.result.unwrap_or(Value::Null));
                    }
                }
            }
        }
    }

    // 导航到指定 URL
    pub fn navigate(&mut self, url: &str) -> Result<()> {
        self.send_command("Page.navigate", serde_json::json!({ "url": url }))?;

        // 等待页面加载完成
        std::thread::sleep(std::time::Duration::from_secs(3));
        Ok(())
    }

    // 执行 JavaScript 并获取结果
    pub fn evaluate(&mut self, script: &str) -> Result<Value> {
        let result = self.send_command(
            "Runtime.evaluate",
            serde_json::json!({
                "expression": script,
                "returnByValue": true,
                "awaitPromise": true
            }),
        )?;

        if let Some(exception) = result.get("exceptionDetails") {
            return Err(anyhow!("JS 执行异常: {:?}", exception));
        }

        Ok(result
            .get("result")
            .and_then(|r| r.get("value"))
            .cloned()
            .unwrap_or(Value::Null))
    }

    pub fn fetch_game_list(&mut self) -> Result<Vec<CloudGameInfo>> {
        log::info!("正在导航到 Steam 云存储页面...");
        self.navigate("https://store.steampowered.com/account/remotestorage")?;

        let script = r#"
            (function() {
                // 只选择 accountTable 下的行
                const rows = Array.from(document.querySelectorAll('.accountTable tr'));
                // 过滤掉表头（通常第一行）和无效行
                return rows.map(tr => {
                    const tds = tr.querySelectorAll('td');
                    // 游戏列表页有4列：游戏名, 文件数, 大小, 链接
                    if(tds.length < 3) return null;
                    
                    // 在行内查找包含 appid 的链接
                    const link = tr.querySelector('a[href*="appid="]');
                    const appIdMatch = link ? link.href.match(/appid=(\d+)/) : null;
                    if(!appIdMatch) return null;
                    
                    return {
                        app_id: parseInt(appIdMatch[1]),
                        game_name: tds[0].textContent.trim(),
                        file_count: parseInt(tds[1].textContent.trim()) || 0,
                        total_size_str: tds[2].textContent.trim()
                    };
                }).filter(x => x);
            })()
        "#;

        let value = self.evaluate(script)?;
        log::debug!("CDP 原始游戏列表数据: {:?}", value);

        let mut games = Vec::new();

        if let Some(arr) = value.as_array() {
            for item in arr {
                let app_id = item["app_id"].as_u64().unwrap_or(0) as u32;
                if app_id == 0 {
                    continue;
                }

                let game_name = item["game_name"]
                    .as_str()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty());
                if let Some(ref name) = game_name {
                    log::debug!("解析到游戏: ID={}, Name={}", app_id, name);
                }

                let file_count = item["file_count"].as_u64().unwrap_or(0) as usize;

                let size_str = item["total_size_str"].as_str().unwrap_or("");
                let total_size = parse_size_str(size_str);
                if total_size == 0 && !size_str.trim().is_empty() {
                    log::warn!(
                        "App {} ({:?}) 大小解析为0: Raw='{}'",
                        app_id,
                        game_name,
                        size_str
                    );
                }

                games.push(CloudGameInfo {
                    app_id,
                    file_count,
                    total_size,
                    last_played: None,
                    playtime: None,
                    game_name,
                    is_installed: false,
                    install_dir: None,
                    categories: Vec::new(),
                });
            }
        }

        Ok(games)
    }

    pub fn fetch_game_files(&mut self, app_id: u32) -> Result<Vec<CloudFile>> {
        let mut all_files = Vec::new();
        let mut page = 1;

        loop {
            let index = (page - 1) * 50;
            let url = if page > 1 {
                format!(
                    "https://store.steampowered.com/account/remotestorageapp/?appid={}&index={}",
                    app_id, index
                )
            } else {
                format!(
                    "https://store.steampowered.com/account/remotestorageapp/?appid={}",
                    app_id
                )
            };

            self.navigate(&url)?;

            let script = r#"
                (function() {
                    const rows = Array.from(document.querySelectorAll('.accountTable tr'));
                    return rows.map(tr => {
                        const tds = tr.querySelectorAll('td');
                        // 文件列表页有 5 列: 文件夹, 文件名, 大小, 时间, 下载链接
                        if(tds.length < 4) return null;
                        
                        // 查找下载链接：可能是 "ugc" 也可能是 "filedownload"
                        const link = tr.querySelector('a[href*="ugc"], a[href*="filedownload"], a[href*="steamusercontent"]');
                        const download_url = link ? link.href : "";
                        
                        // 文件夹, 文件名, 大小, 时间
                        return {
                            folder: tds[0].textContent.trim(),
                            name: tds[1].textContent.trim(),
                            size_str: tds[2].textContent.trim(),
                            time_str: tds[3].textContent.trim(),
                            url: download_url
                        };
                    }).filter(x => x && x.name);
                })()
            "#;

            let value = self.evaluate(script)?;

            if let Some(arr) = value.as_array() {
                if arr.is_empty() {
                    // 如果本页没数据，停止翻页
                }

                for item in arr {
                    let folder = item["folder"].as_str().unwrap_or("").to_string();
                    let filename = item["name"].as_str().unwrap_or("unknown").to_string();
                    let full_name = filename.clone();
                    let size_str = item["size_str"].as_str().unwrap_or("");
                    let time_str = item["time_str"].as_str().unwrap_or("");

                    // 存储 URL 到 root_description，格式: CDP:<URL>|<FOLDER>
                    let url = item["url"].as_str().unwrap_or("").to_string();
                    let root_description = if !url.is_empty() {
                        format!("CDP:{}|{}", url, folder)
                    } else {
                        folder.clone()
                    };

                    let timestamp = parse_steam_time(time_str).unwrap_or_else(Local::now);

                    all_files.push(CloudFile {
                        name: full_name,
                        size: parse_size_str(size_str),
                        timestamp,
                        is_persisted: true,
                        exists: true,
                        root: 0,
                        root_description,
                        conflict: false,
                    });
                }
            }

            // 下一页检测
            let next_index = index + 50;
            // 检查是否存在下一页的链接
            let check_next_script = format!(
                "document.querySelector('a[href*=\"index={}\"]') !== null",
                next_index
            );
            let has_next = self
                .evaluate(&check_next_script)?
                .as_bool()
                .unwrap_or(false);

            if has_next {
                page += 1;
            } else {
                break;
            }
        }

        Ok(all_files)
    }
}

fn parse_steam_time(s: &str) -> Option<chrono::DateTime<Local>> {
    use chrono::TimeZone;
    // Example English: "23 Nov, 2025 @ 5:30pm" or "23 Nov @ 5:30pm"
    // Example Chinese: "2021 年 3 月 6 日 下午 8:04"

    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // 中文格式解析
    if s.contains('年') && s.contains('月') && s.contains('日') {
        let s_clean = s
            .replace("年", " ")
            .replace("月", " ")
            .replace("日", " ")
            .replace(":", " ");
        let parts: Vec<&str> = s_clean.split_whitespace().collect();

        // "2021 3 6 下午 8 04" -> ["2021", "3", "6", "下午", "8", "04"]
        if parts.len() >= 6 {
            let year = parts[0].parse::<i32>().ok()?;
            let month = parts[1].parse::<u32>().ok()?;
            let day = parts[2].parse::<u32>().ok()?;
            let ampm = parts[3];
            let mut hour = parts[4].parse::<u32>().ok()?;
            let minute = parts[5].parse::<u32>().ok()?;

            if (ampm == "下午" || ampm.to_lowercase() == "pm") && hour < 12 {
                hour += 12;
            }
            if (ampm == "上午" || ampm.to_lowercase() == "am") && hour == 12 {
                hour = 0;
            }

            return Local
                .with_ymd_and_hms(year, month, day, hour, minute, 0)
                .single();
        }
    }

    // 英文格式解析
    let s_clean = s.replace(" @", "").replace(",", "");
    let parts: Vec<&str> = s_clean.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let day = parts[0].parse::<u32>().ok()?;
    let month_str = parts[1];
    let month = match month_str.to_lowercase().as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    };

    let mut year = Local::now().year();
    let time_str;

    if parts.len() == 4 {
        year = parts[2].parse::<i32>().ok()?;
        time_str = parts[3];
    } else {
        time_str = parts[2];
    }

    let time_len = time_str.len();
    if time_len < 3 {
        return None;
    }
    let ampm = &time_str[time_len - 2..].to_lowercase();
    let time_val = &time_str[..time_len - 2];
    let time_parts: Vec<&str> = time_val.split(':').collect();
    let mut hour = time_parts[0].parse::<u32>().ok()?;
    let minute = if time_parts.len() > 1 {
        time_parts[1].parse::<u32>().ok()?
    } else {
        0
    };

    if ampm == "pm" && hour < 12 {
        hour += 12;
    }
    if ampm == "am" && hour == 12 {
        hour = 0;
    }

    Local
        .with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
}

fn parse_size_str(s: &str) -> u64 {
    let s = s.replace(",", "").to_lowercase();
    // 处理可能的非标准空格
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
