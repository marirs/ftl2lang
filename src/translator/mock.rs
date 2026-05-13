use super::Translator;
use crate::error::AppError;
use async_trait::async_trait;

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
    ) -> Result<Vec<String>, AppError> {
        Ok(texts.iter().map(|t| t.to_uppercase()).collect())
    }
}
