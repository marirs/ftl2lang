use crate::error::AppError;
use async_trait::async_trait;

pub mod mock;

#[async_trait]
pub trait Translator: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports(&self, target_lang: &str) -> bool;
    async fn translate_batch(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>, AppError>;
}
