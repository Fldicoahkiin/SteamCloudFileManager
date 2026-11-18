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
    load_phosphor_font(&mut fonts);

    ctx.set_fonts(fonts);

    // 应用默认主题
    crate::ui::theme::apply_theme(ctx, crate::ui::theme::ThemeMode::default());
}

// 与 egui_phosphor::add_to_fonts 相同的注册方式。
// egui-phosphor 0.14 依赖 egui 0.36，类型与本项目的 egui 不兼容，所以只取字体字节自行注册
fn load_phosphor_font(fonts: &mut egui::FontDefinitions) {
    const NAME: &str = "phosphor";
    fonts.font_data.insert(
        NAME.to_owned(),
        egui::FontData::from_static(egui_phosphor::Variant::Regular.font_bytes()).into(),
    );

    let mut family = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    family.insert(family.len().min(1), NAME.to_owned());
    fonts
        .families
        .insert(egui::FontFamily::Name(NAME.into()), family);

    if let Some(keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        let at = keys.len().min(1);
        keys.insert(at, NAME.to_owned());
    }
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
