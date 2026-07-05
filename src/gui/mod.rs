// ============================================================================
// gui/mod.rs — GUI module entry
//
// Launches the egui/eframe desktop application.
// ============================================================================

#[cfg(feature = "gui")]
pub mod app;
#[cfg(feature = "gui")]
pub mod pages;
#[cfg(feature = "gui")]
pub mod theme;

#[cfg(feature = "gui")]
pub fn run() {
    use eframe::NativeOptions;
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(1200.0, 800.0))
        .with_min_inner_size(egui::vec2(900.0, 600.0));
    let options = NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Nuwa Backup",
        options,
        Box::new(|_cc| Box::new(app::NuwaApp::new())),
    )
    .expect("Failed to start Nuwa Backup GUI");
}
