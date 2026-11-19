use crate::game_scanner::CloudGameInfo;
use crate::steam_api::CloudFile;
use anyhow::{Result, anyhow};
use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::TcpStream;
use tungstenite::{WebSocket, stream::MaybeTlsStream};

#[derive(Debug, Deserialize)]
struct Target {
    #[serde(rename = "webSocketDebuggerUrl")]
    websocket_debugger_url: Option<String>,
    #[serde(rename = "type")]
    target_type: String,
    url: Option<String>,
    id: Option<String>,
}

pub struct CdpClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    message_id: u64,
    // 由本客户端创建的页面，Drop 时关闭
    owned_target_id: Option<String>,
}

// 本进程新建、尚未关闭的页面。程序退出时后台扫描线程会被直接结束、来不及 Drop，
// 由 close_owned_targets 在退出时统一关闭，避免在 Steam 里留下页面
static OWNED_TARGETS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

pub fn close_owned_targets() {
    let ids = match OWNED_TARGETS.lock() {
        Ok(mut ids) => std::mem::take(&mut *ids),
        Err(_) => return,
    };
    for id in ids {
        close_target(&id);
    }
}

fn close_target(id: &str) {
    if let Ok(mut ids) = OWNED_TARGETS.lock() {
        ids.retain(|owned| owned != id);
    }
    if let Err(e) = cdp_http_agent()
        .put(format!("{}/json/close/{}", CDP_HTTP_BASE, id))
        .send_empty()
    {
        tracing::warn!("关闭 CDP 页面失败: {}", e);
    }
}

impl Drop for CdpClient {
    fn drop(&mut self) {
        let _ = self.socket.close(None);
        if let Some(id) = &self.owned_target_id {
            close_target(id);
        }
    }
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

const CDP_HTTP_BASE: &str = "http://127.0.0.1:8080";
const CDP_HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);
const CDP_SOCKET_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

// 本地调试端口不走代理，并设置超时，避免 Steam 无响应时卡住调用方
fn cdp_http_agent() -> ureq::Agent {
    ureq::config::Config::builder()
        .proxy(None)
        .timeout_global(Some(CDP_HTTP_TIMEOUT))
        .build()
        .new_agent()
}

impl CdpClient {
    // 检查 CDP 服务是否可用
    pub fn is_cdp_running() -> bool {
        static LOG_ONCE: std::sync::Once = std::sync::Once::new();
        let result = cdp_http_agent()
            .get(format!("{}/json/version", CDP_HTTP_BASE))
            .call()
            .is_ok();
        LOG_ONCE.call_once(|| {
            if result {
                tracing::debug!("Steam CEF 调试端口可用 (127.0.0.1:8080)");
            } else {
                tracing::debug!("Steam CEF 调试端口不可用");
            }
        });
        result
    }

    // 连接到一个 store.steampowered.com 页面，之后只在其中执行只读的 fetch，不导航。
    // Steam 自己的商店页带有网页登录态；新建页面不一定有（实测会跳到登录页），所以优先复用。
    // 绝不导航 Steam 自身的页面（SharedJSContext、菜单、弹窗、商店页），否则会破坏 Steam 界面甚至导致闪退。
    // 没有商店页时才新开一个，用完关闭。
    pub fn connect() -> Result<Self> {
        let agent = cdp_http_agent();
        let text = agent
            .get(format!("{}/json", CDP_HTTP_BASE))
            .call()
            .map_err(|e| anyhow!("无法连接到 Steam 调试端口: {}", e))?
            .into_body()
            .read_to_string()?;
        let mut targets: Vec<Target> = serde_json::from_str(&text)?;
        tracing::debug!("CDP 发现 {} 个调试目标", targets.len());

        targets.retain(|t| {
            t.websocket_debugger_url.is_some()
                && t.target_type == "page"
                && t.url
                    .as_ref()
                    .is_some_and(|url| url.starts_with("https://store.steampowered.com/"))
        });
        // 已打开的云存储页优先，其次 Steam 主窗口的商店页，最后其他商店页（如隐藏的营销消息页）
        targets.sort_by_key(|t| {
            let url = t.url.as_deref().unwrap_or("");
            if url.contains("/account/remotestorage") {
                0
            } else if url.contains("marketingmessages") {
                2
            } else {
                1
            }
        });

        let (target, owned_target_id) = match targets.into_iter().next() {
            Some(t) => (t, None),
            None => {
                let created: Target = serde_json::from_str(
                    &agent
                        .put(format!(
                            "{}/json/new?https://store.steampowered.com/",
                            CDP_HTTP_BASE
                        ))
                        .send_empty()
                        .map_err(|e| anyhow!("无法创建 CDP 页面: {}", e))?
                        .into_body()
                        .read_to_string()?,
                )?;
                let id = created.id.clone();
                if let (Some(id), Ok(mut ids)) = (&id, OWNED_TARGETS.lock()) {
                    ids.push(id.clone());
                }
                (created, id)
            }
        };

        tracing::debug!(
            "CDP 连接到目标: url={:?}, 新建={}",
            target.url,
            owned_target_id.is_some()
        );

        let ws_url = target
            .websocket_debugger_url
            .ok_or_else(|| anyhow!("CDP 目标缺少 webSocketDebuggerUrl"))?;
        let connected = tungstenite::connect(&ws_url).map_err(anyhow::Error::from);
        let socket = match connected {
            Ok((socket, _)) => socket,
            Err(e) => {
                if let Some(id) = &owned_target_id {
                    close_target(id);
                }
                return Err(e);
            }
        };
        if let MaybeTlsStream::Plain(stream) = socket.get_ref() {
            stream.set_read_timeout(Some(CDP_SOCKET_TIMEOUT))?;
            stream.set_write_timeout(Some(CDP_SOCKET_TIMEOUT))?;
        }

        let mut client = Self {
            socket,
            message_id: 0,
            owned_target_id,
        };
        if client.owned_target_id.is_some() {
            client.wait_until_loaded();
        }
        Ok(client)
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
        self.socket.send(tungstenite::Message::Text(msg.into()))?;

        loop {
            let msg = self.socket.read()?;
            if let tungstenite::Message::Text(text) = msg
                && let Ok(resp) = serde_json::from_str::<CdpResponse>(&text)
                && resp.id == id
            {
                if let Some(error) = resp.error {
                    return Err(anyhow!("CDP 错误: {:?}", error));
                }
                return Ok(resp.result.unwrap_or(Value::Null));
            }
        }
    }

    // 新建页面需要加载完成后才有同源上下文可用于 fetch
    fn wait_until_loaded(&mut self) {
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(15) {
            if self
                .evaluate("location.origin === 'https://store.steampowered.com' && document.readyState === 'complete'")
                .is_ok_and(|v| v.as_bool() == Some(true))
            {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        tracing::warn!("CDP 新建页面加载超时");
    }

    // 在页面里 fetch 一个商店页面并用 DOMParser 解析，返回 extract(doc) 的结果。
    // 只读：不导航、不改动当前页面。被重定向到登录页时返回明确的错误。
    fn fetch_and_extract(&mut self, path: &str, extract: &str) -> Result<Value> {
        let script = format!(
            r#"(async () => {{
                const r = await fetch({path}, {{ credentials: 'include', signal: AbortSignal.timeout(15000) }});
                if (new URL(r.url).pathname.startsWith('/login')) return {{ login_required: true }};
                const doc = new DOMParser().parseFromString(await r.text(), 'text/html');
                return {{ data: ({extract})(doc) }};
            }})()"#,
            path = serde_json::to_string(path)?,
            extract = extract
        );
        let value = self.evaluate(&script)?;
        if value["login_required"].as_bool() == Some(true) {
            return Err(anyhow!(
                "Steam 网页登录已失效（{} 被重定向到登录页）。请在 Steam 客户端中打开一次「查看 Steam 云」页面后重试",
                path
            ));
        }
        Ok(value["data"].clone())
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
        tracing::info!("正在读取 Steam 云存储页面...");

        let extract = r#"(doc) => Array.from(doc.querySelectorAll('.accountTable tr')).map(tr => {
                const tds = tr.querySelectorAll('td');
                // 游戏列表页有4列：游戏名, 文件数, 大小, 链接
                if (tds.length < 3) return null;
                const link = tr.querySelector('a[href*="appid="]');
                const appIdMatch = link ? (link.getAttribute('href') || '').match(/appid=(\d+)/) : null;
                if (!appIdMatch) return null;
                return {
                    app_id: parseInt(appIdMatch[1]),
                    game_name: tds[0].textContent.trim(),
                    file_count: parseInt(tds[1].textContent.trim()) || 0,
                    total_size_str: tds[2].textContent.trim()
                };
            }).filter(x => x)"#;

        let value = self.fetch_and_extract("/account/remotestorage", extract)?;
        if let Some(arr) = value.as_array() {
            let summary: Vec<String> = arr
                .iter()
                .map(|v| {
                    format!(
                        "{}: {}",
                        v["app_id"].as_u64().unwrap_or(0),
                        v["game_name"].as_str().unwrap_or("unknown")
                    )
                })
                .collect();
            tracing::debug!("CDP 获取到 {} 个游戏: {:?}", arr.len(), summary);
        }

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

                let file_count = item["file_count"].as_u64().unwrap_or(0) as usize;

                let size_str = item["total_size_str"].as_str().unwrap_or("");
                let total_size = crate::file_manager::parse_size(size_str);
                if total_size == 0 && !size_str.trim().is_empty() {
                    tracing::warn!(
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
        } else {
            tracing::warn!("CDP 返回的数据不是数组: {:?}", value);
        }

        tracing::info!("CDP 解析完成，获取到 {} 个游戏", games.len());
        Ok(games)
    }

    pub fn fetch_game_files(&mut self, app_id: u32) -> Result<Vec<CloudFile>> {
        let mut all_files = Vec::new();
        let mut page = 1;

        loop {
            let index = (page - 1) * 50;
            let path = if page > 1 {
                format!(
                    "/account/remotestorageapp/?appid={}&index={}",
                    app_id, index
                )
            } else {
                format!("/account/remotestorageapp/?appid={}", app_id)
            };

            let next_index = index + 50;
            let extract = format!(
                r#"(doc) => ({{
                    rows: Array.from(doc.querySelectorAll('.accountTable tr')).map(tr => {{
                        const tds = tr.querySelectorAll('td');
                        // 文件列表页有 5 列: 文件夹, 文件名, 大小, 时间, 下载链接
                        if (tds.length < 4) return null;
                        const link = tr.querySelector('a[href*="ugc"], a[href*="filedownload"], a[href*="steamusercontent"]');
                        const href = link ? link.getAttribute('href') : '';
                        return {{
                            folder: tds[0].textContent.trim(),
                            name: tds[1].textContent.trim(),
                            size_str: tds[2].textContent.trim(),
                            time_str: tds[3].textContent.trim(),
                            url: href ? new URL(href, 'https://store.steampowered.com/').href : ''
                        }};
                    }}).filter(x => x && x.name),
                    has_next: Array.from(doc.querySelectorAll('a')).some(a => (a.getAttribute('href') || '').includes('index={next_index}'))
                }})"#
            );
            let result = self.fetch_and_extract(&path, &extract)?;
            let value = result["rows"].clone();
            let has_next = result["has_next"].as_bool().unwrap_or(false);

            if let Some(arr) = value.as_array() {
                for item in arr {
                    let folder = item["folder"].as_str().unwrap_or("").to_string();
                    let filename = item["name"].as_str().unwrap_or("unknown").to_string();
                    let size_str = item["size_str"].as_str().unwrap_or("");
                    let time_str = item["time_str"].as_str().unwrap_or("");

                    // 存储 URL 到 root_description，格式: CDP:<URL>|<原始FOLDER>
                    let url = item["url"].as_str().unwrap_or("");
                    let root_description = if !url.is_empty() {
                        format!("CDP:{}|{}", url, folder)
                    } else {
                        folder.clone()
                    };

                    let timestamp = parse_steam_time(time_str).unwrap_or_else(Local::now);

                    all_files.push(CloudFile {
                        name: filename,
                        size: crate::file_manager::parse_size(size_str),
                        timestamp,
                        is_persisted: true,
                        exists: true,
                        root: 0,
                        root_description,
                    });
                }
            }

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
