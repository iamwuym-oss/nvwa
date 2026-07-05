// ============================================================================
// config.rs -- TOML config file read/write module
//
// Responsibilities:
// 1. Read and parse TOML config file (%APPDATA%/nuwa/config.toml)
// 2. Generate default config file (nuwa init)
// 3. Look up job config by job name
//
// Design principles:
// - Config file is the foundation dependency for all Phase 2 features
// - Uses serde deserialization with strict TOML format validation
// - Config file missing/corrupt produces clear errors (no panic)
// ============================================================================

use crate::errors::NuwaError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Top-level config file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Multiple backup job definitions, keyed by job name
    pub job: HashMap<String, JobConfig>,
}

/// Single backup job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    /// Backup source path
    pub source: PathBuf,
    /// Backup destination path
    pub dest: PathBuf,
    /// Whether compression is enabled
    #[serde(default)]
    pub compress: bool,
    /// Retention policy (optional)
    #[serde(default)]
    pub retention: Option<RetentionPolicy>,
}

/// Retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Keep last N backup points
    pub keep_count: Option<u32>,
    /// Keep backup points from last N days
    pub keep_days: Option<u64>,
}

impl Config {
    /// Load config file from default paths
    ///
    /// Search order:
    /// 1. nuwa.toml in current directory
    /// 2. %APPDATA%/nuwa/config.toml
    ///
    /// If neither exists, returns ConfigNotFound error
    pub fn load() -> Result<Self, NuwaError> {
        let paths = Self::config_paths();
        for p in &paths {
            if p.exists() {
                let content = fs::read_to_string(p).map_err(|e| NuwaError::Io {
                    source: Some(e),
                    path: Some(p.clone()),
                    detail: format!("Cannot read config file '{}'", p.display()),
                    suggestion: "Check file permissions and path".to_string(),
                })?;
                let config: Config = toml::from_str(&content).map_err(|e| {
                    NuwaError::ManifestError {
                        detail: format!(
                            "Config file parse failed '{}': {}",
                            p.display(),
                            e.message()
                        ),
                        suggestion: "Check TOML format -- ensure strings use quotes, backslashes in paths are escaped (C:\\\\) or use forward slashes (C:/)"
                            .to_string(),
                    }
                })?;
                return Ok(config);
            }
        }
        Err(NuwaError::InvalidArgument {
            detail: "Config file not found".to_string(),
            suggestion: format!(
                "Run 'nuwa init' first to create a config file.\nSearched paths:\n  {}\n  {}",
                paths[0].display(),
                if paths.len() > 1 {
                    paths[1].display().to_string()
                } else {
                    String::new()
                }
            ),
        })
    }

    /// Load config file from specified path
    pub fn load_from(path: &Path) -> Result<Self, NuwaError> {
        if !path.exists() {
            return Err(NuwaError::InvalidArgument {
                detail: format!("Config file not found '{}'", path.display()),
                suggestion: "Please check the path is correct".to_string(),
            });
        }
        let content = fs::read_to_string(path).map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(path.to_path_buf()),
            detail: format!("Cannot read config file '{}'", path.display()),
            suggestion: "Check file permissions and path".to_string(),
        })?;
        let config: Config = toml::from_str(&content).map_err(|e| NuwaError::ManifestError {
            detail: format!(
                "Config file parse failed '{}': {}",
                path.display(),
                e.message()
            ),
            suggestion: "Check TOML format".to_string(),
        })?;
        Ok(config)
    }

    /// Find a job by name
    ///
    /// If name is None, returns the "default" job
    /// If not found, returns an error
    pub fn find_job(&self, name: Option<&str>) -> Result<(&str, &JobConfig), NuwaError> {
        match name {
            Some(n) => {
                // HashMap::get_key_value returns (&String, &JobConfig)
                // We convert to &str for the return type
                self.job
                    .get_key_value(n)
                    .map(|(k, v)| (k.as_str(), v))
                    .ok_or_else(|| {
                        let available: Vec<&str> = self.job.keys().map(|k| k.as_str()).collect();
                        NuwaError::InvalidArgument {
                            detail: format!("Job '{}' not found", n),
                            suggestion: format!("Available jobs: {}", available.join(", ")),
                        }
                    })
            }
            None => self
                .job
                .get_key_value("default")
                .map(|(k, v)| (k.as_str(), v))
                .ok_or_else(|| NuwaError::InvalidArgument {
                    detail: "No job name specified and no 'default' job found".to_string(),
                    suggestion: "Add [job.default] to config or use --job <name> to specify"
                        .to_string(),
                }),
        }
    }

    /// Get candidate config file paths
    fn config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Current directory
        paths.push(PathBuf::from("nuwa.toml"));

        // 2. APPDATA
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let mut p = PathBuf::from(appdata);
            p.push("nuwa");
            p.push("config.toml");
            paths.push(p);
        }

        paths
    }
}

/// Create a default config file
///
/// Creates the nuwa/ directory and writes the default config at the default path.
/// If the config file already exists, returns an error (to avoid overwriting user config).
pub fn init(config_path: Option<&Path>) -> Result<PathBuf, NuwaError> {
    let config_path = if let Some(path) = config_path {
        // If an explicit path is given, use that directory
        path.to_path_buf()
    } else {
        // Default path: %APPDATA%/nuwa/config.toml
        let appdata = std::env::var_os("APPDATA").ok_or_else(|| NuwaError::InvalidArgument {
            detail: "Cannot determine APPDATA directory".to_string(),
            suggestion: "Specify config path explicitly: nuwa init --config <path>".to_string(),
        })?;
        let mut p = PathBuf::from(appdata);
        p.push("nuwa");
        p.push("config.toml");
        p
    };

    // Check if config already exists
    if config_path.exists() {
        return Err(NuwaError::Io {
            source: None,
            path: Some(config_path.clone()),
            detail: format!("Config file already exists '{}'", config_path.display()),
            suggestion: "Delete the existing config file first if you want to reinitialize"
                .to_string(),
        });
    }

    // Create parent directory
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(parent.to_path_buf()),
            detail: format!("Cannot create directory '{}'", parent.display()),
            suggestion: "Check permissions or create the directory manually".to_string(),
        })?;
    }

    // Default config content
    let default_config = r#"# Nuwa Backup config file
#
# Usage:
#   nuwa backup --job default         Run backup using the 'default' job
#   nuwa backup --job documents        Run backup using the 'documents' job
#   nuwa backup --source <path> --dest <path>  Legacy mode (no config file needed)
#
# Path notes:
# - Escape backslashes in Windows paths: C:\\Users
# - Or use forward slashes: C:/Users
# - Network share paths: \\\\server\\share\\path

[job.default]
source = "C:\\Users"
dest = "D:\\Backup"
compress = false
"#;

    fs::write(&config_path, default_config).map_err(|e| NuwaError::Io {
        source: Some(e),
        path: Some(config_path.clone()),
        detail: format!("Cannot write config file '{}'", config_path.display()),
        suggestion: "Check disk space and permissions".to_string(),
    })?;

    println!("Config file created: {}", config_path.display());
    println!("  Edit this file to add backup jobs, then run:");
    println!("    nuwa backup --job default");

    Ok(config_path)
}

/// Try to load a job with default fallback.
/// This is a helper used by cli.rs when no --job or --source/--dest is specified.
pub fn try_load_job_with_default_fallback(
    name: Option<&str>,
) -> Result<(String, JobConfig), NuwaError> {
    let config = Config::load()?;
    let (name, job) = config.find_job(name)?;
    Ok((name.to_string(), job.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_temp_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("nuwa_config_test_{}_{}", std::process::id(), n))
    }

    fn create_temp_toml(content: &str) -> PathBuf {
        let dir = unique_temp_dir();
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_config.toml");
        fs::write(&path, content).unwrap();
        path
    }

    fn cleanup_temp(path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn test_parse_single_job() {
        let toml = r#"
[job.default]
source = "C:\\Users"
dest = "D:\\Backup"
compress = false
"#;
        let path = create_temp_toml(toml);
        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.job.len(), 1);
        let (name, job) = config.find_job(Some("default")).unwrap();
        assert_eq!(name, "default");
        assert_eq!(job.source.to_str().unwrap(), "C:\\Users");
        assert_eq!(job.dest.to_str().unwrap(), "D:\\Backup");
        assert!(!job.compress);
        assert!(job.retention.is_none());
        cleanup_temp(&path);
    }

    #[test]
    fn test_parse_multi_job() {
        let toml = r#"
[job.default]
source = "C:\\Users"
dest = "D:\\Backup"
compress = false

[job.work]
source = "C:\\Projects"
dest = "E:\\Backup\\Projects"
compress = true

[job.docs]
source = "C:\\Documents"
dest = "\\\\nas\\share\\docs"
compress = true
retention = { keep_count = 10, keep_days = 30 }
"#;
        let path = create_temp_toml(toml);
        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.job.len(), 3);

        let (_, job) = config.find_job(Some("work")).unwrap();
        assert_eq!(job.source.to_str().unwrap(), "C:\\Projects");
        assert!(job.compress);

        let (_, job) = config.find_job(Some("docs")).unwrap();
        assert_eq!(job.dest.to_str().unwrap(), "\\\\nas\\share\\docs");
        let ret = job.retention.as_ref().unwrap();
        assert_eq!(ret.keep_count, Some(10));
        assert_eq!(ret.keep_days, Some(30));

        cleanup_temp(&path);
    }

    #[test]
    fn test_find_default_job() {
        let toml = r#"
[job.default]
source = "C:\\Users"
dest = "D:\\Backup"
compress = false
"#;
        let path = create_temp_toml(toml);
        let config = Config::load_from(&path).unwrap();
        let (name, _) = config.find_job(None).unwrap();
        assert_eq!(name, "default");
        cleanup_temp(&path);
    }

    #[test]
    fn test_job_not_found() {
        let toml = r#"
[job.default]
source = "C:\\Users"
dest = "D:\\Backup"
compress = false
"#;
        let path = create_temp_toml(toml);
        let config = Config::load_from(&path).unwrap();
        let result = config.find_job(Some("nonexistent"));
        assert!(result.is_err());
        cleanup_temp(&path);
    }

    #[test]
    fn test_invalid_toml() {
        let toml = r#"
[job.default]
source = C:\Users
dest = "D:\\Backup"
"#;
        let path = create_temp_toml(toml);
        let result = Config::load_from(&path);
        assert!(result.is_err());
        cleanup_temp(&path);
    }

    #[test]
    fn test_invalid_toml_unclosed_string() {
        let toml = r#"
[job.default]
source = "C:\Users
dest = "D:\\Backup"
"#;
        let path = create_temp_toml(toml);
        let result = Config::load_from(&path);
        assert!(result.is_err());
        cleanup_temp(&path);
    }

    #[test]
    fn test_init_creates_config() {
        let dir = unique_temp_dir();
        let config_path = dir.join("nuwa.toml");
        let result = init(Some(&config_path));
        assert!(result.is_ok());
        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("Nuwa Backup"));
        assert!(content.contains("[job.default]"));
        cleanup_temp(&config_path);
    }

    #[test]
    fn test_init_refuses_existing() {
        let dir = unique_temp_dir();
        let _ = fs::create_dir_all(&dir);
        let config_path = dir.join("nuwa.toml");
        fs::write(&config_path, "existing").unwrap();
        let result = init(Some(&config_path));
        assert!(result.is_err());
        cleanup_temp(&config_path);
    }
}
