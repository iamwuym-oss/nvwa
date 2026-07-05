// ============================================================================
// gui/pages/clone.rs — Clone placeholder (Phase 5 placeholder only)
//
// This page exists only as a future-scope placeholder.
// No clone engine, disk access, partition access, or VSS code exists here.
// ============================================================================

pub const PLACEHOLDER_TEXT: &str =
    "Disk Clone is planned for Phase 5 and is not available in Phase 2.";

pub fn render(ui: &mut egui::Ui) {
    ui.colored_label(egui::Color32::GRAY, PLACEHOLDER_TEXT);
}
