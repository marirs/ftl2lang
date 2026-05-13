use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// On-disk translation cache keyed by `sha256(text|src|tgt|backend)`.
///
/// v3 note: there is no automatic invalidation. If a backend silently
/// changes its output (model upgrade, glossary change, normalization
/// tweak), stale entries are returned forever. The user can work around
/// this by passing `--force` (which bypasses the cache via the pipeline)
/// or by deleting the cache file. A future `--clear-cache` flag or a
/// `version: u32` field on `CacheEntry` would let us invalidate
/// automatically.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TranslationCache {
    pub entries: BTreeMap<String, CacheEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub text: String,
    pub src: String,
    pub tgt: String,
    pub backend: String,
    pub translation: String,
    /// Reserved for future TTL / staleness logic. Currently write-only;
    /// no code path reads it at runtime.
    pub cached_at: String,
}

impl TranslationCache {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str(&text).map_err(|e| AppError::Other(format!("cache parse: {}", e)))
    }

    pub fn save(&self, path: &Path) -> Result<(), AppError> {
        // Guard: only create parent dirs when the path has a non-empty parent component.
        // Same pattern applied in Task 13 sidecar fix — avoids creating "." as a dir.
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let body = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Other(format!("cache serialize: {}", e)))?;
        std::fs::write(path, body)?;
        Ok(())
    }

    pub fn get(&self, text: &str, src: &str, tgt: &str, backend: &str) -> Option<String> {
        let key = key_for(text, src, tgt, backend);
        self.entries.get(&key).map(|e| e.translation.clone())
    }

    pub fn put(&mut self, text: &str, src: &str, tgt: &str, backend: &str, translation: &str) {
        let key = key_for(text, src, tgt, backend);
        self.entries.insert(
            key,
            CacheEntry {
                text: text.into(),
                src: src.into(),
                tgt: tgt.into(),
                backend: backend.into(),
                translation: translation.into(),
                // v3 differs: chrono::Utc::now() used instead of a wall-clock string
                // so timestamps are always RFC-3339 / UTC.
                cached_at: chrono::Utc::now().to_rfc3339(),
            },
        );
    }

    /// Returns the default on-disk cache path: `$CACHE_DIR/ftl2lang/translations.json`.
    pub fn default_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ftl2lang")
            .join("translations.json")
    }
}

/// SHA-256 of `text|src|tgt|backend` — stable, collision-resistant cache key.
fn key_for(text: &str, src: &str, tgt: &str, backend: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    h.update(b"|");
    h.update(src.as_bytes());
    h.update(b"|");
    h.update(tgt.as_bytes());
    h.update(b"|");
    h.update(backend.as_bytes());
    format!("{:x}", h.finalize())
}
