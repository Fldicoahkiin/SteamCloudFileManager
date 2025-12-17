use crate::game_scanner::CloudGameInfo;
use crate::i18n::I18n;
use crate::vdf_parser::UserInfo;
use egui;

// 绘制 About 窗口
// 返回值: (是否需要启动下载, 下载的 release)
pub fn draw_about_window(
    ctx: &egui::Context,
    show: &mut bool,
    about_icon_texture: &mut Option<egui::TextureHandle>,
    update_manager: &mut crate::update::UpdateManager,
    i18n: &I18n,
) -> Option<crate::update::ReleaseInfo> {
    let steam_blue = egui::Color32::from_rgb(102, 192, 244);
    let text_subtle = ctx.style().visuals.text_color().gamma_multiply(0.6);
    let text_normal = ctx.style().visuals.text_color();

    let mut download_release = None;

    egui::Window::new(i18n.about_title())
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
                            // 版本号行
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new("Version")
                                            .size(13.0)
                                            .color(text_subtle),
                                    );
                                },
                            );
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(crate::version::full_version())
                                            .size(13.0)
                                            .color(text_normal)
                                            .monospace(),
                                    );
                                    ui.add_space(8.0);

                                    // 检查更新按钮
                                    let checking = matches!(
                                        update_manager.status(),
                                        crate::update::UpdateStatus::Checking
                                    );
                                    let button_text = if checking {
                                        i18n.checking_update()
                                    } else {
                                        i18n.check_update_btn()
                                    };

                                    if ui
                                        .add_enabled(
                                            !checking,
                                            egui::Button::new(button_text).small(),
                                        )
                                        .clicked()
                                    {
                                        let _ = update_manager.check_update();
                                    }
                                });

                                // 在 Version 行下方显示更新状态提示
                                let update_status = update_manager.status().clone();
                                match &update_status {
                                    crate::update::UpdateStatus::NoUpdate => {
                                        ui.add_space(4.0);
                                        ui.label(
                                            egui::RichText::new(i18n.already_latest())
                                                .size(11.0)
                                                .color(egui::Color32::from_rgb(76, 175, 80)),
                                        );
                                    }
                                    crate::update::UpdateStatus::Available(release) => {
                                        ui.add_space(4.0);
                                        ui.label(
                                            egui::RichText::new(
                                                i18n.new_version_found(&release.tag_name),
                                            )
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(255, 152, 0)),
                                        );
                                    }
                                    crate::update::UpdateStatus::Error(err) => {
                                        ui.add_space(4.0);
                                        ui.label(
                                            egui::RichText::new(format!("❌ {}", err))
                                                .size(10.0)
                                                .color(egui::Color32::from_rgb(244, 67, 54)),
                                        );
                                    }
                                    _ => {}
                                }
                            });
                            ui.end_row();

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

            ui.add_space(16.0);

            // 更新操作区域（仅在有新版本时显示）
            let update_status = update_manager.status().clone();
            if matches!(&update_status, crate::update::UpdateStatus::Available(_)) {
                ui.separator();
                ui.add_space(12.0);

                if let crate::update::UpdateStatus::Available(release) = &update_status {
                    let mut should_open_page = false;

                    ui.vertical_centered(|ui| {
                        #[cfg(target_os = "macos")]
                        ui.label(
                            egui::RichText::new(i18n.new_version_macos_hint())
                                .size(12.0)
                                .color(text_subtle),
                        );

                        #[cfg(not(target_os = "macos"))]
                        ui.label(
                            egui::RichText::new(i18n.new_version_hint())
                                .size(12.0)
                                .color(text_subtle),
                        );
                        ui.add_space(8.0);

                        #[cfg(target_os = "macos")]
                        let button_text = i18n.download_package();

                        #[cfg(not(target_os = "macos"))]
                        let button_text = i18n.download_and_install();

                        if ui.button(button_text).clicked() {
                            download_release = Some(release.clone());
                        }
                        ui.add_space(4.0);
                        if ui.button(i18n.view_details()).clicked() {
                            should_open_page = true;
                        }

                        // 显示下载路径
                        if let Ok(update_dir) = crate::update::UpdateManager::get_update_dir() {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(
                                    i18n.download_location(&update_dir.display().to_string()),
                                )
                                .size(10.0)
                                .color(text_subtle),
                            );
                        }
                    });

                    if should_open_page {
                        crate::update::UpdateManager::open_release_page();
                    }
                }

                ui.add_space(12.0);
            }

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
                                        egui::RichText::new(i18n.author())
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
                                        egui::RichText::new(i18n.github_repository())
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

            // 下载/安装进度显示区域
            let update_status = update_manager.status().clone();
            match &update_status {
                crate::update::UpdateStatus::Downloading(progress) => {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(i18n.downloading_update())
                                .size(13.0)
                                .color(steam_blue),
                        );
                        ui.add_space(8.0);
                        ui.add(egui::ProgressBar::new(*progress).show_percentage());
                    });
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);
                }
                crate::update::UpdateStatus::Installing => {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(i18n.installing_update())
                                .size(13.0)
                                .color(steam_blue),
                        );
                    });
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);
                }
                crate::update::UpdateStatus::Success => {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(i18n.update_success())
                                .size(13.0)
                                .color(egui::Color32::from_rgb(76, 175, 80)),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(i18n.restart_to_apply())
                                .size(11.0)
                                .color(text_subtle),
                        );
                        ui.add_space(8.0);
                        if ui.button(i18n.restart_now()).clicked() {
                            std::process::exit(0);
                        }
                    });
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);
                }
                crate::update::UpdateStatus::Error(err) => {
                    let err_msg = err.clone();
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(format!("❌ {}", err_msg))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(244, 67, 54)),
                        );
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 80.0) / 2.0);
                        if ui.button(i18n.retry()).clicked() {
                            update_manager.reset();
                        }
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);
                }
                _ => {}
            }

            // 日志管理区域
            ui.vertical_centered(|ui| {
                if crate::logger::is_log_config_changed() {
                    let tip_text = if crate::logger::is_log_enabled() {
                        i18n.log_enabled_hint()
                    } else {
                        i18n.log_disabled_hint()
                    };
                    ui.label(
                        egui::RichText::new(tip_text)
                            .size(9.0)
                            .color(egui::Color32::from_rgb(255, 165, 0)),
                    );
                    ui.add_space(8.0);
                }

                let mut log_enabled = crate::logger::is_log_enabled();
                if ui
                    .checkbox(&mut log_enabled, i18n.enable_log_storage())
                    .changed()
                {
                    crate::logger::set_log_enabled(log_enabled);
                    if log_enabled {
                        tracing::info!("日志存储已启用，将在下次启动时生效");
                    } else {
                        tracing::info!("日志存储已禁用，将在下次启动时生效");
                    }
                }

                ui.add_space(12.0);

                if ui.button(i18n.open_log_dir()).clicked() {
                    if let Err(e) = crate::logger::open_log_directory() {
                        tracing::error!("打开日志目录失败: {}", e);
                    }
                }

                if let Ok(log_dir) = crate::logger::get_log_dir() {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(i18n.log_location(&log_dir.display().to_string()))
                            .size(9.0)
                            .color(text_subtle),
                    );
                }
            });

            ui.add_space(10.0);
        });

    download_release
}

// 绘制游戏选择器窗口
pub fn draw_game_selector_window(
    ctx: &egui::Context,
    show: &mut bool,
    games: &[CloudGameInfo],
    is_scanning: bool,
    vdf_count: usize,
    cdp_count: usize,
    i18n: &I18n,
) -> (Option<u32>, bool) {
    let mut selected_app_id = None;
    let mut refresh_clicked = false;

    egui::Window::new(i18n.select_game_title())
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
                        ui.label(i18n.scanning_games());
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
                        ui.label(i18n.no_cloud_games_found());
                    },
                );
            } else {
                ui.horizontal(|ui| {
                    ui.heading(i18n.games_with_cloud(games.len()));
                    // 显示 VDF/CDP 数量
                    ui.label(
                        egui::RichText::new(format!("(VDF: {} | CDP: {})", vdf_count, cdp_count))
                            .color(egui::Color32::GRAY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 刷新按钮
                        if ui
                            .add_enabled(!is_scanning, egui::Button::new(i18n.refresh()))
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
                        if let Some(app_id) = draw_game_item(ui, game, i18n) {
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
fn draw_game_item(ui: &mut egui::Ui, game: &CloudGameInfo, i18n: &I18n) -> Option<u32> {
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
                        ui.colored_label(egui::Color32::from_rgb(0, 200, 0), i18n.installed());
                    } else {
                        ui.colored_label(
                            egui::Color32::from_rgb(150, 150, 150),
                            i18n.not_installed(),
                        );
                    }
                });

                // App ID（如果有游戏名）
                if game.game_name.is_some() {
                    ui.label(format!("App ID: {}", game.app_id));
                }

                // 文件信息
                let file_info = match i18n.language() {
                    crate::i18n::Language::Chinese => format!(
                        "{} 个文件 | {}",
                        game.file_count,
                        crate::file_manager::format_size(game.total_size)
                    ),
                    crate::i18n::Language::English => format!(
                        "{} file{} | {}",
                        game.file_count,
                        if game.file_count != 1 { "s" } else { "" },
                        crate::file_manager::format_size(game.total_size)
                    ),
                };
                ui.label(file_info);

                // 安装目录
                if let Some(dir) = &game.install_dir {
                    let label = match i18n.language() {
                        crate::i18n::Language::Chinese => format!("安装目录: {}", dir),
                        crate::i18n::Language::English => format!("Install dir: {}", dir),
                    };
                    ui.label(label);
                }

                // 标签
                if !game.categories.is_empty() {
                    let label = match i18n.language() {
                        crate::i18n::Language::Chinese => {
                            format!("标签: {}", game.categories.join(", "))
                        }
                        crate::i18n::Language::English => {
                            format!("Tags: {}", game.categories.join(", "))
                        }
                    };
                    ui.label(label);
                }

                // 游戏时间
                if let Some(playtime) = game.playtime {
                    let hours = playtime as f64 / 60.0;
                    let label = match i18n.language() {
                        crate::i18n::Language::Chinese => format!("游戏时间: {:.2} 小时", hours),
                        crate::i18n::Language::English => format!("Playtime: {:.2} hours", hours),
                    };
                    ui.label(label);
                }

                // 最后运行时间
                if let Some(last_played) = game.last_played {
                    if last_played > 0 {
                        use chrono::{DateTime, Local};
                        use std::time::{Duration, UNIX_EPOCH};
                        let dt = UNIX_EPOCH + Duration::from_secs(last_played as u64);
                        let local: DateTime<Local> = dt.into();
                        let label = match i18n.language() {
                            crate::i18n::Language::Chinese => {
                                format!("最后运行: {}", local.format("%Y-%m-%d %H:%M"))
                            }
                            crate::i18n::Language::English => {
                                format!("Last played: {}", local.format("%Y-%m-%d %H:%M"))
                            }
                        };
                        ui.label(label);
                    }
                }
            });

            // 选择按钮
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let button_text = match i18n.language() {
                    crate::i18n::Language::Chinese => "选择",
                    crate::i18n::Language::English => "Select",
                };
                if ui.button(button_text).clicked() {
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
    i18n: &I18n,
) -> Option<String> {
    let mut selected_user_id = None;

    egui::Window::new(i18n.select_user())
        .open(show)
        .resizable(true)
        .default_size([400.0, 300.0])
        .show(ctx, |ui| {
            ui.heading(i18n.steam_users(users.len()));
            ui.add_space(10.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for user in users {
                    if let Some(user_id) = draw_user_item(ui, user, i18n) {
                        selected_user_id = Some(user_id);
                    }
                    ui.add_space(5.0);
                }
            });
        });

    selected_user_id
}

// 绘制单个用户项
fn draw_user_item(ui: &mut egui::Ui, user: &UserInfo, i18n: &I18n) -> Option<String> {
    let mut clicked = false;

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if let Some(name) = &user.persona_name {
                    ui.strong(name);
                    ui.label(format!("ID: {}", user.user_id));
                } else {
                    ui.strong(format!("{}: {}", i18n.user_id(), user.user_id));
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if user.is_current {
                    ui.label(format!("✅ {}", i18n.current_user()));
                } else if ui.button(i18n.switch()).clicked() {
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
pub fn draw_error_window(
    ctx: &egui::Context,
    show: &mut bool,
    error_message: &str,
    i18n: &I18n,
) -> bool {
    let mut confirmed = false;

    egui::Window::new(i18n.error_title())
        .open(show)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(error_message);
            if ui.button(i18n.ok()).clicked() {
                confirmed = true;
            }
        });

    confirmed
}
