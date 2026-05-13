use super::Translator;
use crate::error::AppError;
use async_trait::async_trait;
use indicatif::ProgressBar;

pub struct MockTranslator;

impl MockTranslator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Translator for MockTranslator {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn supports(&self, _target_lang: &str) -> bool {
        true
    }

    async fn translate_batch(
        &self,
        texts: &[&str],
        _source_lang: &str,
        _target_lang: &str,
        progress: Option<&ProgressBar>,
    ) -> Result<Vec<String>, AppError> {
        if let Some(bar) = progress {
            bar.inc(texts.len() as u64);
        }
        Ok(texts.iter().map(|t| t.to_uppercase()).collect())
    }
}
