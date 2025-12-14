use crate::game_scanner::CloudGameInfo;
use crate::vdf_parser::UserInfo;
use egui;

// 绘制 About 窗口
pub fn draw_about_window(
    ctx: &egui::Context,
    show: &mut bool,
    about_icon_texture: &mut Option<egui::TextureHandle>,
) {
    let steam_blue = egui::Color32::from_rgb(102, 192, 244);
    let text_subtle = ctx.style().visuals.text_color().gamma_multiply(0.6);
    let text_normal = ctx.style().visuals.text_color();

    egui::Window::new("About")
        .open(show)
        .resizable(false)
        .collapsible(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.add_space(16.0);

            ui.vertical_centered(|ui| {
                // 加载应用图标
                if about_icon_texture.is_none() {
                    let icon_bytes =
                        include_bytes!("../../assets/steam_cloud-macOS-Default-1024x1024@1x.png");
                    if let Ok(img) = image::load_from_memory(icon_bytes) {
                        let img = img.resize_exact(128, 128, image::imageops::FilterType::Lanczos3);
                        let rgba = img.to_rgba8();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [128, 128],
                            rgba.as_flat_samples().as_slice(),
                        );
                        *about_icon_texture = Some(ui.ctx().load_texture(
                            "about_icon",
                            color_image,
                            Default::default(),
                        ));
                    }
                }

                if let Some(texture) = about_icon_texture.as_ref() {
                    ui.image(texture);
                }

                ui.add_space(16.0);

                ui.label(
                    egui::RichText::new("Steam Cloud File Manager")
                        .size(22.0)
                        .strong()
                        .color(text_normal),
                );
            });

            ui.add_space(24.0);

            ui.horizontal(|ui| {
                let width = ui.available_width();
                let content_width = 320.0;
                ui.add_space((width - content_width) / 2.0);

                ui.vertical(|ui| {
                    ui.set_width(content_width);

                    egui::Grid::new("tech_grid")
                        .num_columns(2)
                        .spacing([16.0, 8.0])
                        .striped(false)
                        .show(ui, |ui| {
                            let mut row = |key: &str, val: String| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(key).size(13.0).color(text_subtle),
                                        );
                                    },
                                );
                                ui.label(
                                    egui::RichText::new(val)
                                        .size(13.0)
                                        .color(text_normal)
                                        .monospace(),
                                );
                                ui.end_row();
                            };

                            row("Version", crate::version::full_version().to_string());
                            row(
                                "OS",
                                format!(
                                    "{} ({})",
                                    crate::version::os_name(),
                                    crate::version::os_arch()
                                ),
                            );
                            row(
                                "Build",
                                format!(
                                    "{} - {}",
                                    crate::version::build_profile(),
                                    crate::version::build_time()
                                ),
                            );
                        });
                });
            });

            ui.add_space(24.0);

            ui.separator();
            ui.add_space(16.0);

            ui.horizontal(|ui| {
                let width = ui.available_width();
                let content_width = 380.0;
                ui.add_space((width - content_width) / 2.0);

                ui.vertical(|ui| {
                    ui.set_width(content_width);

                    egui::Grid::new("links_grid")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new("Author:")
                                            .size(12.0)
                                            .color(text_subtle),
                                    );
                                },
                            );
                            ui.hyperlink_to(
                                egui::RichText::new("Flacier").size(12.0).color(steam_blue),
                                "https://github.com/Fldicoahkiin",
                            );
                            ui.end_row();

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new("Repository:")
                                            .size(12.0)
                                            .color(text_subtle),
                                    );
                                },
                            );
                            ui.hyperlink_to(
                                egui::RichText::new(
                                    "https://github.com/Fldicoahkiin/SteamCloudFileManager",
                                )
                                .size(12.0)
                                .color(steam_blue),
                                "https://github.com/Fldicoahkiin/SteamCloudFileManager",
                            );
                            ui.end_row();
                        });
                });
            });

            ui.add_space(16.0);

            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Copyright © 2025 Flacier")
                        .size(10.0)
                        .color(text_subtle),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("GPL-3.0 License")
                        .size(10.0)
                        .color(text_subtle),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("Powered by Rust & egui")
                        .size(10.0)
                        .color(text_subtle),
                );
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(12.0);

            // 日志管理区域
            ui.vertical_centered(|ui| {
                if crate::logger::is_log_config_changed() {
                    let tip_text = if crate::logger::is_log_enabled() {
                        " 日志存储已启用，重启后生效"
                    } else {
                        " 日志存储已禁用，重启后生效"
                    };
                    ui.label(
                        egui::RichText::new(tip_text)
                            .size(9.0)
                            .color(egui::Color32::from_rgb(255, 165, 0)),
                    );
                    ui.add_space(8.0);
                }

                let mut log_enabled = crate::logger::is_log_enabled();
                if ui.checkbox(&mut log_enabled, "启用日志存储").changed() {
                    crate::logger::set_log_enabled(log_enabled);
                    if log_enabled {
                        tracing::info!("日志存储已启用，将在下次启动时生效");
                    } else {
                        tracing::info!("日志存储已禁用，将在下次启动时生效");
                    }
                }

                ui.add_space(12.0);

                if ui.button(" 打开日志目录").clicked() {
                    if let Err(e) = crate::logger::open_log_directory() {
                        tracing::error!("打开日志目录失败: {}", e);
                    }
                }

                if let Ok(log_dir) = crate::logger::get_log_dir() {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!("日志位置: {}", log_dir.display()))
                            .size(9.0)
                            .color(text_subtle),
                    );
                }
            });

            ui.add_space(10.0);
        });
}

// 绘制游戏选择器窗口
pub fn draw_game_selector_window(
    ctx: &egui::Context,
    show: &mut bool,
    games: &[CloudGameInfo],
    is_scanning: bool,
) -> (Option<u32>, bool) {
    let mut selected_app_id = None;
    let mut refresh_clicked = false;

    egui::Window::new("游戏库")
        .open(show)
        .resizable(true)
        .default_size([600.0, 500.0])
        .show(ctx, |ui| {
            if is_scanning && games.is_empty() {
                // 扫描中且没有游戏，显示居中的加载提示
                let rect = ui.available_rect_before_wrap();
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(rect).layout(
                        egui::Layout::top_down(egui::Align::Center)
                            .with_main_align(egui::Align::Center),
                    ),
                    |ui| {
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.label("正在扫描游戏库...");
                    },
                );
            } else if games.is_empty() {
                let rect = ui.available_rect_before_wrap();
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(rect).layout(
                        egui::Layout::top_down(egui::Align::Center)
                            .with_main_align(egui::Align::Center),
                    ),
                    |ui| {
                        ui.label("未发现云存档的游戏");
                    },
                );
            } else {
                ui.horizontal(|ui| {
                    ui.heading(format!("{} 个有云存档的游戏", games.len()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 刷新按钮
                        if ui
                            .add_enabled(!is_scanning, egui::Button::new("刷新"))
                            .clicked()
                        {
                            refresh_clicked = true;
                        }
                        // 扫描中显示 spinner
                        if is_scanning {
                            ui.spinner();
                        }
                    });
                });
                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for game in games {
                        if let Some(app_id) = draw_game_item(ui, game) {
                            selected_app_id = Some(app_id);
                        }
                        ui.add_space(5.0);
                    }
                });
            }
        });

    (selected_app_id, refresh_clicked)
}

// 绘制单个游戏项
fn draw_game_item(ui: &mut egui::Ui, game: &CloudGameInfo) -> Option<u32> {
    let mut clicked = false;

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                // 游戏名称和状态
                ui.horizontal(|ui| {
                    if let Some(name) = &game.game_name {
                        ui.strong(name);
                    } else {
                        ui.strong(format!("App ID: {}", game.app_id));
                    }

                    if game.is_installed {
                        ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "已安装");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "未安装");
                    }
                });

                // App ID（如果有游戏名）
                if game.game_name.is_some() {
                    ui.label(format!("App ID: {}", game.app_id));
                }

                // 文件信息
                ui.label(format!(
                    "{} 个文件 | {}",
                    game.file_count,
                    crate::utils::format_size(game.total_size)
                ));

                // 安装目录
                if let Some(dir) = &game.install_dir {
                    ui.label(format!("安装目录: {}", dir));
                }

                // 标签
                if !game.categories.is_empty() {
                    ui.label(format!("标签: {}", game.categories.join(", ")));
                }

                // 游戏时间
                if let Some(playtime) = game.playtime {
                    let hours = playtime as f64 / 60.0;
                    ui.label(format!("游戏时间: {:.2} 小时", hours));
                }

                // 最后运行时间
                if let Some(last_played) = game.last_played {
                    if last_played > 0 {
                        use chrono::{DateTime, Local};
                        use std::time::{Duration, UNIX_EPOCH};
                        let dt = UNIX_EPOCH + Duration::from_secs(last_played as u64);
                        let local: DateTime<Local> = dt.into();
                        ui.label(format!("最后运行: {}", local.format("%Y-%m-%d %H:%M")));
                    }
                }
            });

            // 选择按钮
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("选择").clicked() {
                    clicked = true;
                }
            });
        });
    });

    if clicked {
        Some(game.app_id)
    } else {
        None
    }
}

// 绘制用户选择器窗口
pub fn draw_user_selector_window(
    ctx: &egui::Context,
    show: &mut bool,
    users: &[UserInfo],
) -> Option<String> {
    let mut selected_user_id = None;

    egui::Window::new("选择用户")
        .open(show)
        .resizable(true)
        .default_size([400.0, 300.0])
        .show(ctx, |ui| {
            ui.heading(format!("{} 个Steam用户", users.len()));
            ui.add_space(10.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for user in users {
                    if let Some(user_id) = draw_user_item(ui, user) {
                        selected_user_id = Some(user_id);
                    }
                    ui.add_space(5.0);
                }
            });
        });

    selected_user_id
}

// 绘制单个用户项
fn draw_user_item(ui: &mut egui::Ui, user: &UserInfo) -> Option<String> {
    let mut clicked = false;

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if let Some(name) = &user.persona_name {
                    ui.strong(name);
                    ui.label(format!("ID: {}", user.user_id));
                } else {
                    ui.strong(format!("用户 ID: {}", user.user_id));
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if user.is_current {
                    ui.label("✅ 当前用户");
                } else if ui.button("切换").clicked() {
                    clicked = true;
                }
            });
        });
    });

    if clicked {
        Some(user.user_id.clone())
    } else {
        None
    }
}

// 绘制错误窗口
pub fn draw_error_window(ctx: &egui::Context, show: &mut bool, error_message: &str) -> bool {
    let mut confirmed = false;

    egui::Window::new("错误")
        .open(show)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(error_message);
            if ui.button("确定").clicked() {
                confirmed = true;
            }
        });

    confirmed
}
