use crate::error::AppError;
use async_trait::async_trait;

pub mod deepl;
pub mod gtranslate;
pub mod mock;

/// A translation backend (DeepL, Google Cloud, gtranslate, ...).
///
/// The trait is async because every real implementation makes an HTTP request.
/// `Send + Sync` is required so trait objects can cross `.await` points in the
/// pipeline.
#[async_trait]
pub trait Translator: Send + Sync {
    /// Stable backend identifier used in logs, cache keys, and the side-car
    /// file (e.g. `"deepl"`, `"google"`, `"gtranslate"`, `"mock"`).
    fn name(&self) -> &'static str;

    /// Whether this backend supports `target_lang` (an already-normalized
    /// BCP-47 primary tag, e.g. `"de"` or `"pt-BR"`). Implementations should
    /// be permissive when in doubt — gating happens once before any request
    /// is made.
    fn supports(&self, target_lang: &str) -> bool;

    /// Translate a batch of texts from `source_lang` to `target_lang`.
    ///
    /// # Contract
    ///
    /// On `Ok`, the returned `Vec` has exactly `texts.len()` entries, one per
    /// input in the same order. The splice step relies on this length and
    /// order to map translations back to AST spans; an implementation that
    /// filters, reorders, or merges inputs will produce corrupted output.
    async fn translate_batch(
        &self,
        texts: &[&str],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>, AppError>;
}
