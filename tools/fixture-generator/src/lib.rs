//! Deterministic, self-verifying test fixtures for NWB engineering tests.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const DEFAULT_SEED: u64 = 0x4e57_422d_4649_5831;
pub const DATASET_VERSION: u32 = 1;
pub const MANIFEST_NAME: &str = "fixture-manifest.json";

#[derive(Debug, Error)]
pub enum FixtureError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("invalid fixture target: {0}")]
    InvalidTarget(String),
    #[error("invalid fixture manifest: {0}")]
    InvalidManifest(String),
    #[error("fixture verification failed: {0}")]
    Verification(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub dataset_version: u32,
    pub seed: u64,
    pub files: Vec<ManifestEntry>,
    pub root_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestEntry {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

struct DatasetEntry {
    path: &'static str,
    content: Vec<u8>,
}

pub fn generate(root: &Path, seed: u64) -> Result<Manifest, FixtureError> {
    validate_root(root, true)?;
    if root.exists() {
        let mut entries = read_dir(root)?;
        if entries.next().is_some() {
            return Err(FixtureError::InvalidTarget(
                "target directory must be empty".to_owned(),
            ));
        }
    } else {
        fs::create_dir_all(root).map_err(|source| io_error(root, source))?;
    }

    let dataset = expected_dataset(seed);
    for entry in &dataset {
        let output = root.join(entry.path);
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
        }
        write_new_file(&output, &entry.content)?;
    }

    let manifest = manifest_for_dataset(seed, &dataset);
    let manifest_path = root.join(MANIFEST_NAME);
    let mut bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| FixtureError::InvalidManifest(error.to_string()))?;
    bytes.push(b'\n');
    write_new_file(&manifest_path, &bytes)?;
    verify(root)?;
    Ok(manifest)
}

pub fn verify(root: &Path) -> Result<Manifest, FixtureError> {
    validate_root(root, false)?;
    let manifest_path = root.join(MANIFEST_NAME);
    reject_symlink(&manifest_path)?;
    let manifest_bytes =
        fs::read(&manifest_path).map_err(|source| io_error(&manifest_path, source))?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| FixtureError::InvalidManifest(error.to_string()))?;

    if manifest.schema_version != 1 || manifest.dataset_version != DATASET_VERSION {
        return Err(FixtureError::InvalidManifest(
            "unsupported schema_version or dataset_version".to_owned(),
        ));
    }
    validate_manifest_entries(&manifest.files)?;

    let expected = manifest_for_dataset(manifest.seed, &expected_dataset(manifest.seed));
    if manifest != expected {
        return Err(FixtureError::Verification(
            "manifest does not describe the approved dataset for its seed".to_owned(),
        ));
    }

    let actual_paths = collect_files(root)?;
    let expected_paths: BTreeSet<_> = manifest
        .files
        .iter()
        .map(|entry| entry.path.clone())
        .collect();
    if actual_paths != expected_paths {
        return Err(FixtureError::Verification(
            "fixture contains missing or unexpected files".to_owned(),
        ));
    }

    for entry in &manifest.files {
        let path = root.join(&entry.path);
        reject_symlink(&path)?;
        let bytes = fs::read(&path).map_err(|source| io_error(&path, source))?;
        if bytes.len() as u64 != entry.size || sha256_hex(&bytes) != entry.sha256 {
            return Err(FixtureError::Verification(format!(
                "size or SHA-256 mismatch for {}",
                entry.path
            )));
        }
    }
    Ok(manifest)
}

fn expected_dataset(seed: u64) -> Vec<DatasetEntry> {
    vec![
        DatasetEntry {
            path: "boundaries/empty.bin",
            content: Vec::new(),
        },
        DatasetEntry {
            path: "boundaries/one-byte.bin",
            content: vec![0x7f],
        },
        DatasetEntry {
            path: "boundaries/4k.bin",
            content: deterministic_bytes(seed ^ 0x1000, 4 * 1024),
        },
        DatasetEntry {
            path: "boundaries/256k.bin",
            content: deterministic_bytes(seed ^ 0x4_0000, 256 * 1024),
        },
        DatasetEntry {
            path: "patterns/all-zero-64k.bin",
            content: vec![0; 64 * 1024],
        },
        DatasetEntry {
            path: "patterns/repeated-64k.bin",
            content: b"NWB-FIXTURE-"
                .iter()
                .copied()
                .cycle()
                .take(64 * 1024)
                .collect(),
        },
        DatasetEntry {
            path: "random/seeded-64k.bin",
            content: deterministic_bytes(seed, 64 * 1024),
        },
        DatasetEntry {
            path: "deep/a/b/c/d/e/payload.txt",
            content: b"deterministic deep path\n".to_vec(),
        },
        DatasetEntry {
            path: "unicode/\u{6d4b}\u{8bd5}-\u{03b4}-\u{30c7}\u{30fc}\u{30bf}.txt",
            content: b"UTF-8 path fixture\n".to_vec(),
        },
    ]
}

fn deterministic_bytes(seed: u64, length: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(length);
    let mut counter = 0_u64;
    while output.len() < length {
        let mut hasher = Sha256::new();
        hasher.update(b"NWB-FIXTURE-V1");
        hasher.update(seed.to_le_bytes());
        hasher.update(counter.to_le_bytes());
        output.extend_from_slice(&hasher.finalize());
        counter += 1;
    }
    output.truncate(length);
    output
}

fn manifest_for_dataset(seed: u64, dataset: &[DatasetEntry]) -> Manifest {
    let mut files: Vec<_> = dataset
        .iter()
        .map(|entry| ManifestEntry {
            path: entry.path.to_owned(),
            size: entry.content.len() as u64,
            sha256: sha256_hex(&entry.content),
        })
        .collect();
    files.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    let root_sha256 = root_hash(&files);
    Manifest {
        schema_version: 1,
        dataset_version: DATASET_VERSION,
        seed,
        files,
        root_sha256,
    }
}

fn root_hash(files: &[ManifestEntry]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"NWB-FIXTURE-MANIFEST-V1\0");
    for entry in files {
        hasher.update((entry.path.len() as u64).to_le_bytes());
        hasher.update(entry.path.as_bytes());
        hasher.update(entry.size.to_le_bytes());
        hasher.update(entry.sha256.as_bytes());
    }
    hex::encode(hasher.finalize())
}

fn validate_root(root: &Path, may_not_exist: bool) -> Result<(), FixtureError> {
    if root.as_os_str().is_empty() {
        return Err(FixtureError::InvalidTarget(
            "target path must not be empty".to_owned(),
        ));
    }
    if root.exists() {
        reject_symlink(root)?;
        if !root.is_dir() {
            return Err(FixtureError::InvalidTarget(
                "target must be a directory".to_owned(),
            ));
        }
    } else if !may_not_exist {
        return Err(FixtureError::InvalidTarget(
            "fixture directory does not exist".to_owned(),
        ));
    }
    Ok(())
}

fn validate_manifest_entries(entries: &[ManifestEntry]) -> Result<(), FixtureError> {
    let mut previous: Option<&str> = None;
    for entry in entries {
        let path = Path::new(&entry.path);
        if path.is_absolute()
            || path
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
            || entry.path.contains('\\')
        {
            return Err(FixtureError::InvalidManifest(format!(
                "unsafe relative path: {}",
                entry.path
            )));
        }
        if entry.sha256.len() != 64 || hex::decode(&entry.sha256).is_err() {
            return Err(FixtureError::InvalidManifest(format!(
                "invalid SHA-256 for {}",
                entry.path
            )));
        }
        if previous.is_some_and(|value| value.as_bytes() >= entry.path.as_bytes()) {
            return Err(FixtureError::InvalidManifest(
                "file entries must be unique and bytewise sorted".to_owned(),
            ));
        }
        previous = Some(&entry.path);
    }
    Ok(())
}

fn collect_files(root: &Path) -> Result<BTreeSet<String>, FixtureError> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = BTreeSet::new();
    while let Some(directory) = pending.pop() {
        for item in read_dir(&directory)? {
            let item = item.map_err(|source| io_error(&directory, source))?;
            let path = item.path();
            let metadata = fs::symlink_metadata(&path).map_err(|source| io_error(&path, source))?;
            if metadata.file_type().is_symlink() {
                return Err(FixtureError::Verification(format!(
                    "symbolic links are forbidden: {}",
                    path.display()
                )));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() && path.file_name() != Some(OsStr::new(MANIFEST_NAME)) {
                let relative = path.strip_prefix(root).map_err(|_| {
                    FixtureError::Verification("file escaped fixture root".to_owned())
                })?;
                let normalized = relative
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                files.insert(normalized);
            }
        }
    }
    Ok(files)
}

fn reject_symlink(path: &Path) -> Result<(), FixtureError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| io_error(path, source))?;
    if metadata.file_type().is_symlink() {
        return Err(FixtureError::InvalidTarget(format!(
            "symbolic links are forbidden: {}",
            path.display()
        )));
    }
    Ok(())
}

fn read_dir(path: &Path) -> Result<fs::ReadDir, FixtureError> {
    fs::read_dir(path).map_err(|source| io_error(path, source))
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), FixtureError> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options
        .open(path)
        .map_err(|source| io_error(path, source))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|source| io_error(path, source))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn io_error(path: &Path, source: io::Error) -> FixtureError {
    FixtureError::Io {
        path: path.to_path_buf(),
        source,
    }
}
