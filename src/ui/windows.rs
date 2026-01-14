use crate::game_scanner::CloudGameInfo;
use crate::i18n::I18n;
use crate::icons;
use crate::vdf_parser::UserInfo;
use egui;

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
                            .color(crate::ui::theme::muted_color(ui.ctx())),
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
                        ui.colored_label(
                            crate::ui::theme::installed_color(ui.ctx()),
                            i18n.installed(),
                        );
                    } else {
                        ui.colored_label(
                            crate::ui::theme::muted_color(ui.ctx()),
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
                if let Some(last_played) = game.last_played
                    && last_played > 0
                {
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

    if clicked { Some(game.app_id) } else { None }
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
                    ui.label(format!("{} {}", icons::CHECK, i18n.current_user()));
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
