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
        let text = std::fs::read_to_string(path)?;
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

pub fn sidecar_path_for(target_ftl: &Path) -> PathBuf {
    let mut p = target_ftl.as_os_str().to_owned();
    p.push(".ftl2lang.json");
    PathBuf::from(p)
}
