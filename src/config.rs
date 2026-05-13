use crate::error::AppError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub default_translator: Option<String>,
    pub default_source: Option<String>,
    pub deepl: Option<DeeplConfig>,
    pub google: Option<GoogleConfig>,
    pub gtranslate: Option<GtranslateConfig>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct DeeplConfig {
    pub api_key: Option<String>,
    pub api_url: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct GoogleConfig {
    pub api_key: Option<String>,
    pub project_id: Option<String>,
}

// GtranslateConfig is currently empty; fields will be added when the
// Google Translate (free scraper) backend is implemented in Task 12.
#[derive(Debug, Default, Deserialize, Clone)]
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
        let text = std::fs::read_to_string(path)
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
