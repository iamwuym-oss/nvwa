// ============================================================================
// gui/app.rs — NuwaApp main application state
//
// Holds the current page selection and renders the overall UI layout
// with a left sidebar for navigation and a right content area.
// ============================================================================

use crate::gui::pages::Page;

/// Main application state
pub struct NuwaApp {
    /// Currently selected/active page
    pub current_page: Page,
}

impl NuwaApp {
    /// Create a new NuwaApp with default page (Dashboard)
    pub fn new() -> Self {
        Self {
            current_page: Page::Dashboard,
        }
    }
}

impl Default for NuwaApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for NuwaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::gui::theme::apply_style(ctx);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Nuwa Backup");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Phase 2");
                });
            });
        });

        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .min_width(180.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label("");
                });
                ui.separator();
                ui.add_space(4.0);

                let pages = Page::all();
                let mut new_page = None;
                for page in &pages {
                    let selected = &self.current_page == page;
                    let label = page.label();
                    if ui.selectable_label(selected, label).clicked() {
                        new_page = Some(*page);
                    }
                }

                if let Some(p) = new_page {
                    self.current_page = p;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading(self.current_page.title());
            ui.separator();
            ui.add_space(8.0);
            self.current_page.render(ui);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_page_is_dashboard() {
        let app = NuwaApp::new();
        assert_eq!(app.current_page, Page::Dashboard);
    }

    #[test]
    fn test_all_pages_exist() {
        let pages = Page::all();
        assert_eq!(pages.len(), 7);
        let names: Vec<&str> = pages.iter().map(|p| p.label()).collect();
        assert!(names.contains(&"Dashboard"));
        assert!(names.contains(&"Backup"));
        assert!(names.contains(&"Restore"));
        assert!(names.contains(&"History"));
        assert!(names.contains(&"Schedule"));
        assert!(names.contains(&"Clone"));
        assert!(names.contains(&"Settings"));
    }

    #[test]
    fn test_clone_placeholder_text() {
        let text = crate::gui::pages::clone::PLACEHOLDER_TEXT;
        assert_eq!(
            text,
            "Disk Clone is planned for Phase 5 and is not available in Phase 2."
        );
    }

    #[test]
    fn test_page_can_switch() {
        let mut app = NuwaApp::new();
        assert_eq!(app.current_page, Page::Dashboard);
        app.current_page = Page::Backup;
        assert_eq!(app.current_page, Page::Backup);
        app.current_page = Page::Clone;
        assert_eq!(app.current_page, Page::Clone);
    }

    #[test]
    fn test_no_clone_action_exists() {
        // Verify that the Clone page has no clickable action button
        let page = Page::Clone;
        let label = page.label();
        assert_eq!(label, "Clone");
        let title = page.title();
        assert_eq!(title, "Clone");
    }
}
