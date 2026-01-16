use egui;

// 内嵌 Sarasa UI SC 字体
// 来源: https://github.com/be5invis/Sarasa-Gothic
// 许可证: SIL Open Font License 1.1
static SARASA_UI_SC: &[u8] = include_bytes!("../../assets/fonts/SarasaUiSC-Regular.ttf");

// 设置应用字体
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 加载内嵌的 Sarasa UI SC 字体
    load_embedded_font(&mut fonts);

    // 加载 Phosphor 图标字体
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    ctx.set_fonts(fonts);

    // 应用默认主题
    crate::ui::theme::apply_theme(ctx, crate::ui::theme::ThemeMode::default());
}

// 加载内嵌字体
fn load_embedded_font(fonts: &mut egui::FontDefinitions) {
    fonts.font_data.insert(
        "sarasa_ui_sc".to_owned(),
        egui::FontData::from_static(SARASA_UI_SC).into(),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "sarasa_ui_sc".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "sarasa_ui_sc".to_owned());

    tracing::info!("已加载内嵌字体: SarasaUiSC-Regular.ttf");
}
