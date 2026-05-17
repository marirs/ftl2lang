use crate::error::AppError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub default_translator: Option<String>,
    pub default_source: Option<String>,
    pub deepl: Option<DeeplConfig>,
    pub google: Option<GoogleConfig>,
    pub gtranslate: Option<GtranslateConfig>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DeeplConfig {
    pub api_key: Option<String>,
    pub api_url: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct GoogleConfig {
    pub api_key: Option<String>,
    pub project_id: Option<String>,
}

// GtranslateConfig is currently empty; fields will be added when the
// Google Translate (free scraper) backend is implemented in Task 12.
// `non_exhaustive` so callers can't construct it directly and break
// when fields are added.
#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct GtranslateConfig {}

impl Config {
    /// Load config from an explicit path.
    ///
    /// Returns `Ok(Config::default())` when the file does not exist so that
    /// callers do not need to handle "first run, no config yet" as an error.
    /// Returns `Err` only for I/O failures on an existing file or for TOML
    /// parse errors.
    pub fn load_from_path(path: &Path) -> Result<Self, AppError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        // Size-capped read: a hostile or corrupted --config target can't
        // make us swallow gigabytes before toml::from_str rejects it.
        let text = crate::fsutil::read_to_string_capped(path)
            .map_err(|e| AppError::Config(format!("reading {}: {}", path.display(), e)))?;
        toml::from_str(&text)
            .map_err(|e| AppError::Config(format!("parsing {}: {}", path.display(), e)))
    }

    /// Canonical config file location: `~/.config/ftl2lang/config.toml`.
    ///
    /// Falls back to `./config.toml` when the platform config directory
    /// cannot be determined (unusual but possible in minimal containers).
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ftl2lang")
            .join("config.toml")
    }
}
