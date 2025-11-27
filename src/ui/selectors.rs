use crate::game_scanner::CloudGameInfo;
use crate::vdf_parser::UserInfo;
use egui;

// 绘制游戏选择器窗口
pub fn draw_game_selector_window(
    ctx: &egui::Context,
    show: &mut bool,
    games: &[CloudGameInfo],
    is_scanning: bool,
) -> Option<u32> {
    let mut selected_app_id = None;

    egui::Window::new("游戏库")
        .open(show)
        .resizable(true)
        .default_size([600.0, 500.0])
        .show(ctx, |ui| {
            if is_scanning && games.is_empty() {
                ui.label("正在扫描游戏库...");
            } else if games.is_empty() {
                ui.label("未发现云存档的游戏");
            } else {
                ui.heading(format!("{} 个有云存档的游戏", games.len()));
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

    selected_app_id
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
