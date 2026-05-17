use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Sidecar {
    #[serde(flatten)]
    pub entries: BTreeMap<String, SidecarEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SidecarEntry {
    pub src_hash: String,
    /// RFC3339 timestamp string. v3 note: stored as String for simplicity; if
    /// ordering or filtering by date is ever needed, switch to chrono::DateTime.
    pub translated_at: String,
    pub backend: String,
}

impl Sidecar {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        // Size-capped read so a runaway side-car can't OOM us.
        let text = crate::fsutil::read_to_string_capped(path)?;
        Self::from_json(&text)
    }

    pub fn save(&self, path: &Path) -> Result<(), AppError> {
        // Atomic write: a crash mid-write can no longer leave a truncated
        // side-car. atomic_write also handles creating the parent directory.
        crate::fsutil::atomic_write(path, &self.to_json()?)
    }

    pub fn from_json(s: &str) -> Result<Self, AppError> {
        serde_json::from_str(s).map_err(|e| AppError::Other(format!("sidecar parse: {}", e)))
    }

    pub fn to_json(&self) -> Result<String, AppError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Other(format!("sidecar serialize: {}", e)))
    }
}

pub fn hash_text(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    format!("sha256:{:x}", h.finalize())
}

/// Build the path of the side-car JSON file that sits next to `target_ftl`.
///
/// `target_ftl` must be a file path (e.g. `de.ftl`). If it ends in a path
/// separator the result would be a hidden file *inside* the directory
/// (`<dir>/.ftl2lang.json`), which is almost certainly not what the caller
/// wants — that case is explicitly rejected via the trailing-separator check.
pub fn sidecar_path_for(target_ftl: &Path) -> PathBuf {
    // Strip any trailing separator and reject the empty / directory-shaped
    // case so a misuse panics loudly during development rather than
    // silently writing the side-car in the wrong place.
    let s = target_ftl.as_os_str();
    let bytes = s.as_encoded_bytes();
    debug_assert!(
        !bytes.is_empty()
            && !bytes.ends_with(b"/")
            && (cfg!(not(windows)) || !bytes.ends_with(b"\\")),
        "sidecar_path_for: target_ftl must be a file path, got {:?}",
        target_ftl
    );
    let mut p = s.to_owned();
    p.push(".ftl2lang.json");
    PathBuf::from(p)
}
