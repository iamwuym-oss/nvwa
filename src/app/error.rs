// ============================================================================
// error.rs -- Application Layer error types
//
// AppError wraps core NuwaError variants into a clean API contract that the
// Tauri command layer can serialize and return to the React frontend.
//
// Design:
// - Each variant has a category so React can switch on error type
// - Message is user-facing
// - No stack traces or internal details leak to the UI
// ============================================================================

use serde::{Deserialize, Serialize};
use std::fmt;

/// Application-layer error with category tagging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    /// Error category -- React uses this for conditional UI
    pub category: String,
    /// User-facing error message
    pub message: String,
    /// Internal detail (serialized only in debug builds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl AppError {
    pub fn config(msg: impl Into<String>) -> Self {
        Self {
            category: "Config".into(),
            message: msg.into(),
            detail: None,
        }
    }

    pub fn history(msg: impl Into<String>) -> Self {
        Self {
            category: "History".into(),
            message: msg.into(),
            detail: None,
        }
    }

    pub fn storage(msg: impl Into<String>) -> Self {
        Self {
            category: "Storage".into(),
            message: msg.into(),
            detail: None,
        }
    }

    pub fn permission(msg: impl Into<String>) -> Self {
        Self {
            category: "Permission".into(),
            message: msg.into(),
            detail: None,
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            category: "Internal".into(),
            message: msg.into(),
            detail: None,
        }
    }

    /// Attach an internal detail (for debug logging, not shown to users by default)
    pub fn with_detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.category, self.message)
    }
}

impl std::error::Error for AppError {}

/// Convert core NuwaError into AppError
impl From<crate::errors::NuwaError> for AppError {
    fn from(e: crate::errors::NuwaError) -> Self {
        match &e {
            // Config file not found = invalid argument with "Config file not found" detail
            crate::errors::NuwaError::InvalidArgument { detail, .. }
                if detail == "Config file not found" =>
            {
                AppError::config("No configuration found. Create a backup job to get started.")
            }
            // Manifest/config parse errors = config category
            crate::errors::NuwaError::ManifestError { .. } => AppError::config(e.to_string()),
            // I/O errors = storage category
            crate::errors::NuwaError::Io { detail, .. } => AppError::storage(detail.clone()),
            // All other errors = internal
            _ => AppError::internal(e.to_string()),
        }
    }
}
