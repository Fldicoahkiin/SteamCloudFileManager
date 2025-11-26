mod app;
mod cdp_client;
mod file_manager;
mod game_scanner;
mod path_resolver;
mod steam_api;
mod user_manager;
mod utils;
mod vdf_parser;

use app::SteamCloudApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let env = env_logger::Env::default().filter_or(
        "RUST_LOG",
        "info,SteamCloudFileManager=debug,ureq=warn,rustls=warn,tungstenite=warn",
    );
    env_logger::init_from_env(env);

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
