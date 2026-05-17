use super::deepl::DeeplTranslator;
use super::google::GoogleTranslator;
use super::gtranslate::GtranslateTranslator;
use super::Translator;
use crate::config::Config;
use crate::error::AppError;

/// Build a translator implementation from a `--translator` flag value (None
/// to use the config default, or "gtranslate" if no config default either).
///
/// `verbose` enables per-request diagnostic logging inside the backend.
///
/// Returns `MissingApiKey` when the requested backend needs configuration
/// that is not present.
pub fn build_translator(
    name: Option<&str>,
    config: &Config,
    verbose: bool,
) -> Result<Box<dyn Translator>, AppError> {
    let resolved: String = name
        .map(|s| s.to_string())
        .or_else(|| config.default_translator.clone())
        .unwrap_or_else(|| "gtranslate".to_string());

    match resolved.as_str() {
        "gtranslate" => Ok(Box::new(GtranslateTranslator::new().with_verbose(verbose))),
        "deepl" => {
            // Distinguish "section missing" from "key missing" so users see
            // an actionable hint about whether to add the section header or
            // just fill in the value.
            let cfg = config
                .deepl
                .as_ref()
                .ok_or_else(|| AppError::MissingApiKey {
                    backend: "deepl".into(),
                    field: "[deepl] section".into(),
                })?;
            let key = cfg.api_key.clone().ok_or_else(|| AppError::MissingApiKey {
                backend: "deepl".into(),
                field: "[deepl].api_key".into(),
            })?;
            Ok(Box::new(
                DeeplTranslator::new(key, cfg.api_url.clone()).with_verbose(verbose),
            ))
        }
        "google" => {
            let cfg = config
                .google
                .as_ref()
                .ok_or_else(|| AppError::MissingApiKey {
                    backend: "google".into(),
                    field: "[google] section".into(),
                })?;
            let key = cfg.api_key.clone().ok_or_else(|| AppError::MissingApiKey {
                backend: "google".into(),
                field: "[google].api_key".into(),
            })?;
            let project = cfg
                .project_id
                .clone()
                .ok_or_else(|| AppError::MissingApiKey {
                    backend: "google".into(),
                    field: "[google].project_id".into(),
                })?;
            Ok(Box::new(
                GoogleTranslator::new(key, project).with_verbose(verbose),
            ))
        }
        other => Err(AppError::Other(format!(
            "unknown translator '{}'. Valid: deepl | google | gtranslate.",
            other
        ))),
    }
}
