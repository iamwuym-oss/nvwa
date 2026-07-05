// ============================================================================
// gui/pages/mod.rs — Page enum and page rendering dispatch
// ============================================================================

pub mod backup;
pub mod clone;
pub mod dashboard;
pub mod history;
pub mod restore;
pub mod schedule;
pub mod settings;

/// All navigable pages in the Nuwa Backup GUI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Page {
    Dashboard,
    Backup,
    Restore,
    History,
    Schedule,
    Clone,
    Settings,
}

impl Page {
    /// Return all pages in sidebar display order
    pub fn all() -> Vec<Page> {
        vec![
            Page::Dashboard,
            Page::Backup,
            Page::Restore,
            Page::History,
            Page::Schedule,
            Page::Clone,
            Page::Settings,
        ]
    }

    /// Short label shown in the left navigation sidebar
    pub fn label(&self) -> &'static str {
        match self {
            Page::Dashboard => "Dashboard",
            Page::Backup => "Backup",
            Page::Restore => "Restore",
            Page::History => "History",
            Page::Schedule => "Schedule",
            Page::Clone => "Clone",
            Page::Settings => "Settings",
        }
    }

    /// Page title displayed in the content header
    pub fn title(&self) -> &'static str {
        match self {
            Page::Dashboard => "Dashboard",
            Page::Backup => "Backup",
            Page::Restore => "Restore",
            Page::History => "History",
            Page::Schedule => "Schedule",
            Page::Clone => "Clone",
            Page::Settings => "Settings",
        }
    }

    /// Render the page content into the given egui UI
    pub fn render(&self, ui: &mut egui::Ui) {
        match self {
            Page::Dashboard => dashboard::render(ui),
            Page::Backup => backup::render(ui),
            Page::Restore => restore::render(ui),
            Page::History => history::render(ui),
            Page::Schedule => schedule::render(ui),
            Page::Clone => clone::render(ui),
            Page::Settings => settings::render(ui),
        }
    }
}
