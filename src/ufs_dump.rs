// UFS 配置工具模块
// 提供获取和输出 UFS 配置的功能
// cargo run -- --ufs <appid>

use crate::vdf_parser::VdfParser;

// 获取指定 AppID 的原始 UFS 配置文本
pub fn get_ufs_raw_text(app_id: u32) -> Result<String, String> {
    let parser = VdfParser::new().map_err(|e| format!("初始化失败: {}", e))?;
    let config = parser
        .get_ufs_config(app_id)
        .map_err(|e| format!("获取配置失败: {}", e))?;
    Ok(config.raw_text)
}

// 输出指定 AppID 的 UFS 配置到标准输出
pub fn dump(app_id_str: &str) {
    let app_id: u32 = match app_id_str.parse() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("无效的 AppID: {}", app_id_str);
            return;
        }
    };

    match get_ufs_raw_text(app_id) {
        Ok(text) => println!("{}", text),
        Err(e) => eprintln!("{}", e),
    }
}
