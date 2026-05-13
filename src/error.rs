use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("error: {backend} backend requires {field} in ~/.config/ftl2lang/config.toml")]
    MissingApiKey { backend: String, field: String },

    #[error("error: {backend} does not support '{lang}'. Try --translator {suggestion}.")]
    UnsupportedLang {
        backend: String,
        lang: String,
        suggestion: String,
    },

    #[error("error: failed to parse {path}: {message}")]
    FtlParse { path: PathBuf, message: String },

    #[error("error: translation API failed: {0}")]
    Api(String),

    #[error("error: I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("error: config: {0}")]
    Config(String),

    #[error("error: {0}")]
    Other(String),
}

pub fn exit_code(err: &AppError) -> i32 {
    match err {
        AppError::MissingApiKey { .. } => 2,
        AppError::FtlParse { .. } => 3,
        AppError::UnsupportedLang { .. } => 4,
        AppError::Api(_) => 5,
        AppError::Config(_) => 6,
        AppError::Io(_) => 7,
        AppError::Other(_) => 1,
    }
}
