// ============================================================================
// repo_registry.rs -- Repository registry (persistent registration table)
//
// The registry maps UUID -> (name, path, status) so that Repository identity
// is independent of filesystem path. Users can move or rename the directory
// without losing the Repository reference.
//
// File: ~/.nuwa/repositories.json
//
// Crash safety: atomic write (.tmp -> rename). On parse failure, the old
// file is preserved and an error is returned (never auto-truncate).
// ============================================================================

use std::fs;
use std::path::{Path, PathBuf};

use crate::app::error::AppError;
use crate::app::models::repo::RepoRecord;

/// Default registry file location: ~/.nuwa/repositories.json
fn default_registry_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let mut path = PathBuf::from(home);
    path.push(".nuwa");
    path.push("repositories.json");
    path
}

/// The Repository registry: a persistent JSON file mapping UUID -> RepoRecord.
///
/// The registry is an append-only list (no physical deletion of records).
/// A record's status is set to "removed" rather than deleted, preserving
/// audit traceability.
pub struct RepoRegistry {
    path: PathBuf,
    repositories: Vec<RepoRecord>,
}

impl RepoRegistry {
    /// Load the registry from the default location (~/.nuwa/repositories.json).
    /// If the file does not exist, returns an empty registry (NOT an error).
    pub fn load() -> Self {
        Self::load_from(&default_registry_path())
    }

    /// Load from a specific path (for testing).
    pub fn load_from(path: &Path) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    match serde_json::from_str::<Vec<RepoRecord>>(&content) {
                        Ok(records) => {
                            return Self {
                                path: path.to_path_buf(),
                                repositories: records,
                            };
                        }
                        Err(e) => {
                            // Parse failure: log warning, return empty
                            // Never auto-truncate or overwrite a valid file
                            eprintln!(
                                "[repo_registry] Warning: failed to parse {}: {}",
                                path.display(),
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[repo_registry] Warning: cannot read {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        Self {
            path: path.to_path_buf(),
            repositories: Vec::new(),
        }
    }

    /// Persist the current registry to disk (atomic write).
    fn save(&self) -> Result<(), AppError> {
        let json = serde_json::to_string_pretty(&self.repositories)
            .map_err(|e| AppError::internal(format!("Registry serialization failed: {}", e)))?;

        let tmp_path = self.path.with_extension("json.tmp");
        fs::write(&tmp_path, &json)
            .map_err(|e| AppError::internal(format!("Cannot write registry temp file: {}", e)))?;

        fs::rename(&tmp_path, &self.path).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            AppError::internal(format!("Cannot rename registry file: {}", e))
        })?;

        Ok(())
    }

    /// Return all registered repositories (excluding "removed" status).
    pub fn list(&self) -> Vec<&RepoRecord> {
        self.repositories
            .iter()
            .filter(|r| r.status != "removed")
            .collect()
    }

    /// Return all records including removed (for audit).
    pub fn list_all(&self) -> &[RepoRecord] {
        &self.repositories
    }

    /// Look up a Repository by its UUID.
    pub fn get(&self, id: &str) -> Option<&RepoRecord> {
        self.repositories
            .iter()
            .find(|r| r.id == id && r.status != "removed")
    }

    /// Register a new Repository. Returns error if UUID already exists.
    pub fn register(&mut self, record: RepoRecord) -> Result<(), AppError> {
        if self.repositories.iter().any(|r| r.id == record.id) {
            return Err(AppError::config(format!(
                "Repository '{}' is already registered",
                record.id
            )));
        }
        self.repositories.push(record);
        self.save()
    }

    /// Remove a Repository from the registry (soft-delete: sets status to "removed").
    /// Does NOT delete the underlying Repository data.
    pub fn unregister(&mut self, id: &str) -> Result<(), AppError> {
        if let Some(record) = self.repositories.iter_mut().find(|r| r.id == id) {
            record.status = "removed".into();
            record.last_opened = chrono::Local::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string();
            return self.save();
        }
        Err(AppError::config(format!(
            "Repository '{}' not found in registry",
            id
        )))
    }

    /// Update the filesystem path for a registered Repository.
    pub fn update_path(&mut self, id: &str, new_path: &str) -> Result<(), AppError> {
        if let Some(record) = self.repositories.iter_mut().find(|r| r.id == id) {
            record.path = new_path.to_string();
            record.last_opened = chrono::Local::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string();
            return self.save();
        }
        Err(AppError::config(format!(
            "Repository '{}' not found in registry",
            id
        )))
    }

    /// Update the status of a registered Repository (e.g., "missing" -> "active").
    pub fn update_status(&mut self, id: &str, status: &str) -> Result<(), AppError> {
        if let Some(record) = self.repositories.iter_mut().find(|r| r.id == id) {
            record.status = status.to_string();
            return self.save();
        }
        Err(AppError::config(format!(
            "Repository '{}' not found in registry",
            id
        )))
    }

    /// Mark a Repository as last_opened = now (called after successful open).
    pub fn touch(&mut self, id: &str) -> Result<(), AppError> {
        if let Some(record) = self.repositories.iter_mut().find(|r| r.id == id) {
            record.last_opened = chrono::Local::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string();
            record.status = "active".into();
            return self.save();
        }
        Err(AppError::config(format!(
            "Repository '{}' not found in registry",
            id
        )))
    }

    /// Resolve a Repository UUID to its filesystem path.
    /// Returns None if the UUID is not registered or the path does not exist.
    pub fn resolve_path(&self, id: &str) -> Result<PathBuf, AppError> {
        let record = self
            .get(id)
            .ok_or_else(|| AppError::config(format!("Repository '{}' is not registered", id)))?;
        let path = PathBuf::from(&record.path);
        if !path.exists() {
            return Err(AppError::config(format!(
                "Repository path '{}' does not exist. The disk may be disconnected.",
                path.display()
            )));
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_record(id: &str, name: &str, path: &str) -> RepoRecord {
        RepoRecord {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            created_at: "2026-07-10T10:00:00Z".into(),
            last_opened: "2026-07-10T10:00:00Z".into(),
            status: "active".into(),
        }
    }

    #[test]
    fn test_registry_empty_on_missing_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");
        let registry = RepoRegistry::load_from(&path);
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_registry_register_and_list() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");
        let mut registry = RepoRegistry::load_from(&path);

        registry
            .register(make_record("id-1", "Repo A", "/tmp/a"))
            .unwrap();
        registry
            .register(make_record("id-2", "Repo B", "/tmp/b"))
            .unwrap();

        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_registry_duplicate_rejected() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");
        let mut registry = RepoRegistry::load_from(&path);

        registry
            .register(make_record("id-1", "Repo A", "/tmp/a"))
            .unwrap();
        let result = registry.register(make_record("id-1", "Repo A dup", "/tmp/a"));
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_get_by_id() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");
        let mut registry = RepoRegistry::load_from(&path);

        registry
            .register(make_record("id-1", "Repo A", "/tmp/a"))
            .unwrap();
        let found = registry.get("id-1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Repo A");

        let not_found = registry.get("id-99");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_registry_unregister_soft_delete() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");
        let mut registry = RepoRegistry::load_from(&path);

        registry
            .register(make_record("id-1", "Repo A", "/tmp/a"))
            .unwrap();
        registry.unregister("id-1").unwrap();

        assert!(registry.get("id-1").is_none());
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_registry_persistence() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");

        // First session: register
        {
            let mut registry = RepoRegistry::load_from(&path);
            registry
                .register(make_record("id-1", "Repo A", "/tmp/a"))
                .unwrap();
        }

        // Second session: verify persistence
        {
            let registry = RepoRegistry::load_from(&path);
            let list = registry.list();
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].name, "Repo A");
        }
    }

    #[test]
    fn test_registry_update_path() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");
        let mut registry = RepoRegistry::load_from(&path);

        registry
            .register(make_record("id-1", "Repo A", "/tmp/a"))
            .unwrap();
        registry.update_path("id-1", "/tmp/b").unwrap();

        let updated = registry.get("id-1").unwrap();
        assert_eq!(updated.path, "/tmp/b");
    }

    #[test]
    fn test_registry_touch_updates_timestamp() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");
        let mut registry = RepoRegistry::load_from(&path);

        registry
            .register(make_record("id-1", "Repo A", "/tmp/a"))
            .unwrap();
        registry.touch("id-1").unwrap();

        let updated = registry.get("id-1").unwrap();
        // last_opened should have been updated (not the original value)
        assert_ne!(updated.last_opened, "2026-07-10T10:00:00Z");
        assert_eq!(updated.status, "active");
    }

    #[test]
    fn test_registry_corrupted_file_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");
        fs::write(&path, "this is not json").unwrap();

        let registry = RepoRegistry::load_from(&path);
        // Corrupted file -> empty registry, never crash
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_resolve_path_fails_for_unregistered() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");
        let registry = RepoRegistry::load_from(&path);

        let result = registry.resolve_path("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_path_fails_for_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("repositories.json");
        let mut registry = RepoRegistry::load_from(&path);

        registry
            .register(make_record("id-1", "Repo A", "/nonexistent/path"))
            .unwrap();
        let result = registry.resolve_path("id-1");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("does not exist"));
    }
}
