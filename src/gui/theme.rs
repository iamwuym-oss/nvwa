// ============================================================================
// gui/theme.rs — Basic visual theme for Nuwa Backup GUI
// ============================================================================

/// Apply the Nuwa Backup visual style to the egui context
pub fn apply_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.window_rounding = 4.0.into();
    style.visuals.panel_fill = egui::Color32::from_rgb(30, 30, 35);
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(40, 40, 48);
    ctx.set_style(style);
}
