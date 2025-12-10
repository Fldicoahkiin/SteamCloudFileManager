#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod cdp_client;
mod error;
mod file_manager;
mod file_tree;
mod game_scanner;
mod logger;
mod path_resolver;
mod steam_api;
mod steam_process;
mod ui;
mod user_manager;
mod utils;
mod vdf_parser;
mod version;

use app::SteamCloudApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // 初始化日志系统（输出到文件和控制台）
    if let Err(e) = logger::init_logger() {
        eprintln!("日志初始化失败: {}", e);
        // 降级到只输出到控制台
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    "info,SteamCloudFileManager=debug,ureq=warn,rustls=warn,tungstenite=warn".into()
                }),
            )
            .with_target(true)
            .with_thread_ids(false)
            .with_line_number(true)
            .init();
    }

    // 打印版本信息
    tracing::info!("\n{}", version::version_info());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_app_id("com.flacier.steamcloudfilemanager")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Steam 云文件管理器 - Steam Cloud File Manager",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_embed_viewports(false);
            Ok(Box::new(SteamCloudApp::new(cc)))
        }),
    )
}

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../assets/steam_cloud-macOS-Default-1024x1024@1x.png");
    match image::load_from_memory(icon_bytes) {
        Ok(image) => {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            }
        }
        Err(_) => egui::IconData {
            rgba: vec![255; 32 * 32 * 4],
            width: 32,
            height: 32,
        },
    }
}
